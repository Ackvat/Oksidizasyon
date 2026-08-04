#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut nodespace = Nodespace::new();
        let testnode_id = nodespace.add_node(String::from("testnode"));
        
        if let Some(node) = nodespace.get_node_mut(testnode_id) {
            node.name = String::from("testnode_1");
            println!("Node name: {:?}", node);
        }
        
        println!("Parent name: {:?}", nodespace);
    }
}

use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Nodespace {
    pub nodes: Vec<Option<Node>>,
}

impl Nodespace {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
    
    // Creates a node inside the nodespace.
    pub fn add_node(&mut self, name: String) -> usize {
        let id = self.nodes.len();
        let mut node = Node::new(name);
        node.id = id;
        self.nodes.push(Some(node));
        id
    }

    pub fn remove_node(&mut self, id: usize) -> Option<Node> {
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

    pub fn get_node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id)?.as_ref()
    }

    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node> {
        self.nodes.get_mut(id)?.as_mut()
    }
}



#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,

    pub name: String,
}

impl Node {
    pub fn new(name: String) -> Self {
        Self {
            id: 0,
            parent: None,
            children: Vec::new(),
            name,
        }
    }

    pub fn with_parent(mut self, parent_id: Option<usize>) {
        self.parent = parent_id;
    } 
}