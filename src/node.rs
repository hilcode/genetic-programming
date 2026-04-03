use std::fmt;

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn leaf(name: impl Into<String>) -> Node {
        Node { name: name.into(), children: Vec::new() }
    }

    pub fn branch(name: impl Into<String>, children: Vec<Node>) -> Node {
        Node { name: name.into(), children }
    }

    pub fn size(&self) -> usize {
        1 + self.children.iter().map(|child| child.size()).sum::<usize>()
    }

    pub fn get(&self, index: usize) -> &Node {
        self.get_internal(index, 0).0.expect("index out of bounds")
    }

    fn get_internal(&self, target_index: usize, current: usize) -> (Option<&Node>, usize) {
        if current == target_index {
            return (Some(self), current + self.size());
        }
        let mut counter: usize = current + 1;
        for child in &self.children {
            let (found, after): (Option<&Node>, usize) = child.get_internal(target_index, counter);
            if found.is_some() {
                return (found, after);
            }
            counter = after;
        }
        (None, counter)
    }

    pub fn replace(&self, index: usize, replacement: &Node) -> Node {
        self.replace_internal(index, replacement, 0).0
    }

    fn replace_internal(&self, target_index: usize, replacement: &Node, current: usize) -> (Node, usize) {
        if current == target_index {
            return (replacement.clone(), current + self.size());
        }
        let mut counter: usize = current + 1;
        let mut new_children: Vec<Node> = Vec::with_capacity(self.children.len());
        for child in &self.children {
            let (new_child, after): (Node, usize) =
                child.replace_internal(target_index, replacement, counter);
            counter = after;
            new_children.push(new_child);
        }
        (Node::branch(self.name.clone(), new_children), counter)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.children.is_empty() {
            write!(formatter, "{}", self.name)
        } else {
            write!(formatter, "({}", self.name)?;
            for child in &self.children {
                write!(formatter, " {}", child)?;
            }
            write!(formatter, ")")
        }
    }
}
