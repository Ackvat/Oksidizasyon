#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        //let mut nodespace = Node::new(String::from("nodespace"));
        //let mut testnode = Node::new(String::from("testnode")).with_parent(Some(&mut nodespace));
        println!("Node name: {:?}, Parent name: {:?}", testnode, nodespace);
    }
}

//use crate::{NODE_SIZE};

use alloc::{string::String, vec::Vec};

/* Deprecated, still saving it in case of a need of returning back.
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
}
*/

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub parent: Option<*mut Node>,
    pub children: Vec<Option<*mut Node>>,

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
}