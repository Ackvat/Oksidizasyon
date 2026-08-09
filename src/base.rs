#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut nodespace: Nodespace<Component> = Nodespace::new();
        let testnode_id = nodespace.add_node(String::from("testnode"), None);
        
        if let Some(node) = nodespace.get_node_mut(testnode_id) {
            node.name = String::from("testnode_1");
            println!("Node name: {:?}", node);
        }
        
        println!("Parent name: {:?}", nodespace);
    }
}

use alloc::{string::String, vec::Vec};


#[derive(Debug)]
pub enum Component {

}

impl Component {

}

#[derive(Debug, Clone)]
pub struct Nodespace<C> {
    pub nodes: Vec<Option<Node<C>>>,
}

impl<C> Nodespace<C> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
    
    // Creates a node inside the nodespace.
    pub fn add_node(&mut self, name: String, parent_id: Option<usize>) -> usize {
        let id = self.nodes.len();
        let mut node = Node::new(name);
        node.id = id;
        self.nodes.push(Some(node));
        id
    }

    pub fn remove_node(&mut self, id: usize) -> Option<Node<C>> {
        let node = self.nodes.get_mut(id)?.take()?;

        if let Some(p) = node.parent {
            if let Some(pn) = self.get_node_mut(p) {
                pn.children.retain(|&c| c != id);
            }
        }

        for &c in &node.children {
            if let Some(cn) = self.get_node_mut(c) {
                cn.parent = None;
            }
        }

        Some(node)
    }

    pub fn get_node(&self, id: usize) -> Option<&Node<C>> {
        self.nodes.get(id)?.as_ref()
    }

    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node<C>> {
        self.nodes.get_mut(id)?.as_mut()
    }

    pub fn find_node(&self, name: String) -> Option<&Node<C>> {
        self.nodes.iter().flatten().find(|n| n.name == name)
    }

    pub fn set_node_parent(&mut self, node_id: usize, parent_id: Option<usize>) -> Option<&Node<C>>{
        let node = self.nodes.get_mut(node_id)?;

        // Detach existing parent, if there is one.
        if let Some(p) = node.parent {
            if let Some(pn) = self.get_node_mut(p) {
                pn.children.retain(|&c| c != node.id);
            }
        }

        // Set the parent_id
        node.parent = parent_id;

        // Return the current parent just in case it is needed.
        Some(self.get_node(parent_id))
    }
}

//impl<C> Default for Nodespace<C> {
//    todo!()
//}



#[derive(Debug, Clone)]
pub struct Node<C> {
    pub id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,

    pub name: String,

    pub components: Vec<C>,
}

impl<C> Node<C> {
    pub fn new(name: String) -> Self {
        Self {
            id: 0,
            parent: None,
            children: Vec::new(),
            name,
            components: Vec::new(),
        }
    }

    pub fn attach(&mut self, component: C) -> &mut C {
        self.components.push(component);
        self.components.last_mut().unwrap()
    }

    pub fn with_parent(&mut self, parent_id: Option<usize>) {
        self.parent = parent_id;
    }
}


#[derive(Debug)] pub struct SinkError;
pub trait Sink {
    fn write(&mut self, bytes: &[u8]) -> Result<(), SinkError>;
}

pub trait Update {
    fn update(&mut self, dt: f32);
}



pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

pub struct Logger {
    buf: [u8; 256],
    head: usize,
    tail: usize,
    pub level: LogLevel,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            buf: [0; 256],
            head: 0,
            tail: 0,
            level: LogLevel::TRACE,
        }
    }
}

impl Sink for Logger {
    fn write(&mut self, bytes: &[u8]) -> Result<(), SinkError> {
        for &b in bytes {
            self.buf[self.head % 256] = b;
            self.head += 1;
        }
        Ok(())
    }
}



pub struct UARTPort {
    peripheral: u8,
    baud: u32,
    tx_dropped: u32, // Bytes lost to a full FIFO.
}

impl UARTPort {
    pub fn new(peripheral: u8, baud: u32) -> Self {
        Self {
            peripheral,
            baud,
            tx_dropped: 0
        }
    }
}

impl Sink for UARTPort {
    fn write(&mut self, bytes: &[u8]) -> Result<(), SinkError> {
        todo!()
    }
}
