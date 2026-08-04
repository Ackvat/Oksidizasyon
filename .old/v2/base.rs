// The base abstraction: a parent/child object tree that works the same on a Pi 4 and a Pico.
//
// Python's `Base` holds `self.parent` and `self.children` directly on the object. Rust cannot own a
// graph that way without reference counting, and reference counting needs an allocator the Pico
// does not have. So the tree is inverted: a `Registry` owns every node in one fixed-size array, and
// nodes refer to each other through `NodeId` handles instead of pointers. Nothing here allocates,
// nothing here is `unsafe`, and the whole structure can live in a `static`.
//
// Children are an intrusive doubly-linked list (`first_child` + `next_sibling`) rather than a list
// per node. That keeps attach and detach O(1), imposes no per-node child limit, and costs the same
// few bytes whether a node has one child or forty. Insertion order is preserved, which matters when
// the children are motor 1..4.

// A handle to a node. The generation guards against the classic arena bug: a node is despawned, its
// slot is reused by an unrelated node, and a stale handle silently starts addressing the new one.
// Bumping the slot's generation on free makes every old handle compare unequal, so the lookup fails
// loudly instead. Generations wrap after 65536 reuses of the same slot; a handle held across that
// many reuses of one slot is already a bug of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId {
    index: u16,
    generation: u16,
}

// Every tree operation reports its failure rather than panicking. On a flight controller a bad
// handle should degrade one subsystem, not take down the whole board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseError {
    // The registry is at capacity; raise `N`.
    Full,
    // The handle points at a freed or reused slot.
    Stale,
    // The move would make a node its own ancestor.
    Cycle,
}

// The link fields, split out so `Base` reads as identity plus payload.
#[derive(Debug, Clone, Copy)]
struct Links {
    parent: Option<NodeId>,
    first_child: Option<NodeId>,
    // Tracked purely so appending a child is O(1) instead of a walk to the end of the sibling list.
    last_child: Option<NodeId>,
    next_sibling: Option<NodeId>,
    prev_sibling: Option<NodeId>,
}

impl Links {
    const fn new() -> Self {
        Self {
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
        }
    }
}

// A node in the tree. `T` is whatever the caller is modelling: a motor, an IMU, a coordinate frame.
// One tree implementation serves all of them, and the payload comes back typed.
#[derive(Debug)]
pub struct Base<T> {
    pub name: &'static str,
    pub value: T,
    links: Links,
}

impl<T> Base<T> {
    pub fn parent(&self) -> Option<NodeId> {
        self.links.parent
    }

    pub fn first_child(&self) -> Option<NodeId> {
        self.links.first_child
    }

    pub fn last_child(&self) -> Option<NodeId> {
        self.links.last_child
    }

    pub fn next_sibling(&self) -> Option<NodeId> {
        self.links.next_sibling
    }

    pub fn prev_sibling(&self) -> Option<NodeId> {
        self.links.prev_sibling
    }
}

// A vacant slot carries the next link of the free list, so recycling a slot is a pop rather than a
// scan for a hole.
enum Entry<T> {
    Vacant { next_free: Option<u16> },
    Occupied(Base<T>),
}

struct Slot<T> {
    generation: u16,
    entry: Entry<T>,
}

// Owns every node. `N` is the hard capacity, fixed at compile time because the Pico has no heap to
// grow into; sizing it is the caller's decision and running out is a normal, reported error.
pub struct Registry<T, const N: usize> {
    slots: [Slot<T>; N],
    // Head of the free list of previously used slots.
    free_head: Option<u16>,
    // Slots beyond this index have never been used, so they need no free-list entry. This is what
    // lets `new` be `const`: every slot starts identical.
    high_water: u16,
    count: u16,
}

impl<T, const N: usize> Registry<T, N> {
    pub const fn new() -> Self {
        assert!(
            N <= u16::MAX as usize,
            "Registry capacity must fit in a u16 index.",
        );

        Self {
            slots: [const {
                Slot {
                    generation: 0,
                    entry: Entry::Vacant { next_free: None },
                }
            }; N],
            free_head: None,
            high_water: 0,
            count: 0,
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn len(&self) -> usize {
        self.count as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.node(id).is_some()
    }

    // Creates a detached node. It has no parent until it is attached to one.
    pub fn spawn(&mut self, name: &'static str, value: T) -> Result<NodeId, BaseError> {
        let index = self.alloc_slot()?;
        let slot = &mut self.slots[index as usize];

        slot.entry = Entry::Occupied(Base {
            name,
            value,
            links: Links::new(),
        });
        self.count += 1;

        Ok(NodeId {
            index,
            generation: slot.generation,
        })
    }

    // Spawns a node already attached to `parent`, which is the common case.
    pub fn spawn_child(
        &mut self,
        parent: NodeId,
        name: &'static str,
        value: T,
    ) -> Result<NodeId, BaseError> {
        // Checked before the slot is taken, so a bad parent does not leave an orphan behind.
        self.links(parent)?;

        let child = self.spawn(name, value)?;
        self.attach(parent, child)?;

        Ok(child)
    }

    pub fn get(&self, id: NodeId) -> Option<&Base<T>> {
        self.node(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Base<T>> {
        self.node_mut(id)
    }

    pub fn value(&self, id: NodeId) -> Option<&T> {
        self.node(id).map(|node| &node.value)
    }

    pub fn value_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.node_mut(id).map(|node| &mut node.value)
    }

    pub fn name_of(&self, id: NodeId) -> Option<&'static str> {
        self.node(id).map(|node| node.name)
    }

    pub fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?.links.parent
    }

    pub fn first_child_of(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?.links.first_child
    }

    pub fn next_sibling_of(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?.links.next_sibling
    }

    // Appends `child` to `parent`, detaching it from any previous parent first.
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) -> Result<(), BaseError> {
        self.attach(parent, child)
    }

    // `None` releases the node to the top level, matching `Set_Parent(None)` in the Python original.
    pub fn set_parent(&mut self, child: NodeId, parent: Option<NodeId>) -> Result<(), BaseError> {
        match parent {
            Some(parent) => self.attach(parent, child),
            None => self.release(child),
        }
    }

    // Unlinks a node from its parent without destroying it.
    pub fn release(&mut self, id: NodeId) -> Result<(), BaseError> {
        self.links(id)?;
        self.detach(id)
    }

    // Direct children only, mirroring `Find_Child`. For a deep search, filter `descendants`.
    pub fn find_child(&self, parent: NodeId, name: &str) -> Option<NodeId> {
        self.children(parent).find(|&id| self.name_of(id) == Some(name))
    }

    // Detaches the named child and hands the caller its handle; the node stays alive and parentless.
    pub fn remove_child(&mut self, parent: NodeId, name: &str) -> Option<NodeId> {
        let child = self.find_child(parent, name)?;
        self.detach(child).ok()?;

        Some(child)
    }

    // Destroys a node and everything under it, freeing every slot for reuse.
    //
    // The walk is iterative on purpose. A recursive teardown would put tree depth on the stack, and
    // the Pico's stack is measured in kilobytes; a deep transform hierarchy would overflow it.
    pub fn despawn(&mut self, root: NodeId) -> Result<(), BaseError> {
        self.links(root)?;
        // Unlink first, so the surrounding sibling list stays intact while the subtree is torn down.
        self.detach(root)?;

        let mut cursor = root;
        loop {
            // Always descend the first child, so the node being freed is always a leaf and the only
            // link left pointing at it is its parent's `first_child`.
            if let Some(child) = self.links(cursor)?.first_child {
                cursor = child;
                continue;
            }

            let links = *self.links(cursor)?;
            self.free_slot(cursor);

            let Some(parent) = links.parent else {
                // Only the released root has no parent, so the subtree is gone.
                return Ok(());
            };

            let parent_links = self.links_mut(parent)?;
            parent_links.first_child = links.next_sibling;
            if links.next_sibling.is_none() {
                parent_links.last_child = None;
            }
            if let Some(next) = links.next_sibling {
                self.links_mut(next)?.prev_sibling = None;
            }

            cursor = parent;
        }
    }

    // Direct children, in insertion order.
    pub fn children(&self, id: NodeId) -> Children<'_, T, N> {
        Children {
            registry: self,
            next: self.first_child_of(id),
        }
    }

    // Walks parent links upward, nearest ancestor first. Excludes the node itself.
    pub fn ancestors(&self, id: NodeId) -> Ancestors<'_, T, N> {
        Ancestors {
            registry: self,
            next: self.parent_of(id),
        }
    }

    // The whole subtree below `id` in depth-first pre-order, excluding `id` itself. This is the one
    // to fold a parent transform down through a kinematic chain.
    pub fn descendants(&self, id: NodeId) -> Descendants<'_, T, N> {
        Descendants {
            registry: self,
            root: id,
            next: self.first_child_of(id),
        }
    }

    pub fn root_of(&self, id: NodeId) -> Option<NodeId> {
        self.node(id)?;

        let mut cursor = id;
        while let Some(parent) = self.parent_of(cursor) {
            cursor = parent;
        }

        Some(cursor)
    }

    fn node(&self, id: NodeId) -> Option<&Base<T>> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }

        match &slot.entry {
            Entry::Occupied(node) => Some(node),
            Entry::Vacant { .. } => None,
        }
    }

    fn node_mut(&mut self, id: NodeId) -> Option<&mut Base<T>> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.generation != id.generation {
            return None;
        }

        match &mut slot.entry {
            Entry::Occupied(node) => Some(node),
            Entry::Vacant { .. } => None,
        }
    }

    fn links(&self, id: NodeId) -> Result<&Links, BaseError> {
        self.node(id).map(|node| &node.links).ok_or(BaseError::Stale)
    }

    fn links_mut(&mut self, id: NodeId) -> Result<&mut Links, BaseError> {
        self.node_mut(id)
            .map(|node| &mut node.links)
            .ok_or(BaseError::Stale)
    }

    fn alloc_slot(&mut self) -> Result<u16, BaseError> {
        if let Some(index) = self.free_head {
            let next_free = match &self.slots[index as usize].entry {
                Entry::Vacant { next_free } => *next_free,
                // The free list only ever chains vacant slots; an occupied one means the registry
                // has been corrupted, so refuse rather than hand out a live slot twice.
                Entry::Occupied(_) => return Err(BaseError::Full),
            };
            self.free_head = next_free;

            return Ok(index);
        }

        if (self.high_water as usize) < N {
            let index = self.high_water;
            self.high_water += 1;

            return Ok(index);
        }

        Err(BaseError::Full)
    }

    // Assumes the node is already unlinked from the tree; `despawn` is the only caller.
    fn free_slot(&mut self, id: NodeId) {
        let next_free = self.free_head;
        let slot = &mut self.slots[id.index as usize];

        slot.generation = slot.generation.wrapping_add(1);
        // Replacing the entry drops the payload, so `T`'s own cleanup still runs.
        slot.entry = Entry::Vacant { next_free };

        self.free_head = Some(id.index);
        self.count -= 1;
    }

    fn attach(&mut self, parent: NodeId, child: NodeId) -> Result<(), BaseError> {
        if parent == child {
            return Err(BaseError::Cycle);
        }
        self.links(parent)?;
        self.links(child)?;
        // Reparenting a node under its own descendant would splice a ring out of the tree and hang
        // every later traversal, so it is rejected before anything is written.
        if self.is_ancestor(child, parent)? {
            return Err(BaseError::Cycle);
        }

        self.detach(child)?;

        match self.links(parent)?.last_child {
            Some(last) => {
                self.links_mut(last)?.next_sibling = Some(child);
                self.links_mut(child)?.prev_sibling = Some(last);
            }
            None => {
                self.links_mut(parent)?.first_child = Some(child);
            }
        }

        self.links_mut(parent)?.last_child = Some(child);
        self.links_mut(child)?.parent = Some(parent);

        Ok(())
    }

    fn detach(&mut self, id: NodeId) -> Result<(), BaseError> {
        let links = *self.links(id)?;

        let Some(parent) = links.parent else {
            return Ok(());
        };

        match links.prev_sibling {
            Some(prev) => self.links_mut(prev)?.next_sibling = links.next_sibling,
            None => self.links_mut(parent)?.first_child = links.next_sibling,
        }
        match links.next_sibling {
            Some(next) => self.links_mut(next)?.prev_sibling = links.prev_sibling,
            None => self.links_mut(parent)?.last_child = links.prev_sibling,
        }

        let links = self.links_mut(id)?;
        links.parent = None;
        links.prev_sibling = None;
        links.next_sibling = None;

        Ok(())
    }

    fn is_ancestor(&self, candidate: NodeId, of: NodeId) -> Result<bool, BaseError> {
        let mut cursor = self.links(of)?.parent;
        let mut steps = 0;

        while let Some(node) = cursor {
            if node == candidate {
                return Ok(true);
            }

            // No chain of parents can be longer than the registry, so exceeding it means an existing
            // cycle. Bailing out keeps a corrupt tree from spinning here forever.
            steps += 1;
            if steps > N {
                return Err(BaseError::Cycle);
            }

            cursor = self.links(node)?.parent;
        }

        Ok(false)
    }
}

impl<T, const N: usize> Default for Registry<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Children<'a, T, const N: usize> {
    registry: &'a Registry<T, N>,
    next: Option<NodeId>,
}

impl<T, const N: usize> Iterator for Children<'_, T, N> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.registry.next_sibling_of(current);

        Some(current)
    }
}

pub struct Ancestors<'a, T, const N: usize> {
    registry: &'a Registry<T, N>,
    next: Option<NodeId>,
}

impl<T, const N: usize> Iterator for Ancestors<'_, T, N> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.registry.parent_of(current);

        Some(current)
    }
}

pub struct Descendants<'a, T, const N: usize> {
    registry: &'a Registry<T, N>,
    root: NodeId,
    next: Option<NodeId>,
}

impl<T, const N: usize> Descendants<'_, T, N> {
    // Pre-order successor: down first, then across, then up and across. Climbing stops at the
    // subtree root so the walk never escapes into the root's own siblings.
    fn step(&self, from: NodeId) -> Option<NodeId> {
        if let Some(child) = self.registry.first_child_of(from) {
            return Some(child);
        }

        let mut cursor = from;
        loop {
            if cursor == self.root {
                return None;
            }
            if let Some(sibling) = self.registry.next_sibling_of(cursor) {
                return Some(sibling);
            }

            cursor = self.registry.parent_of(cursor)?;
        }
    }
}

impl<T, const N: usize> Iterator for Descendants<'_, T, N> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.step(current);

        Some(current)
    }
}
