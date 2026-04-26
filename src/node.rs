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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_creation() {
        let leaf: Node = Node::leaf("x");
        assert_eq!(leaf.name, "x");
        assert_eq!(leaf.children.len(), 0);
    }

    #[test]
    fn test_branch_creation() {
        let left: Node = Node::leaf("a");
        let right: Node = Node::leaf("b");
        let branch: Node = Node::branch("+", vec![left, right]);
        assert_eq!(branch.name, "+");
        assert_eq!(branch.children.len(), 2);
    }

    #[test]
    fn test_size() {
        let leaf: Node = Node::leaf("x");
        assert_eq!(leaf.size(), 1);

        let branch: Node = Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]);
        assert_eq!(branch.size(), 3); // 1 root + 2 leaves

        let nested: Node = Node::branch(
            "*",
            vec![
                Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]),
                Node::leaf("c"),
            ],
        );
        assert_eq!(nested.size(), 5); // 1 root + 3 (left subtree) + 1 leaf
    }

    #[test]
    fn test_get() {
        // Tree: (* (+ a b) c)
        //       0  1  2 3  4
        let tree: Node = Node::branch(
            "*",
            vec![
                Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]),
                Node::leaf("c"),
            ],
        );

        assert_eq!(tree.get(0).name, "*");
        assert_eq!(tree.get(1).name, "+");
        assert_eq!(tree.get(2).name, "a");
        assert_eq!(tree.get(3).name, "b");
        assert_eq!(tree.get(4).name, "c");
    }

    #[test]
    fn test_replace() {
        // Tree: (* (+ a b) c)
        let tree: Node = Node::branch(
            "*",
            vec![
                Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]),
                Node::leaf("c"),
            ],
        );

        // Replace 'a' at index 2 with 'x'
        let replacement: Node = Node::leaf("x");
        let new_tree: Node = tree.replace(2, &replacement);
        assert_eq!(new_tree.get(2).name, "x");
        assert_eq!(new_tree.to_string(), "(* (+ x b) c)");

        // Replace entire subtree at index 1 with 'y'
        let new_tree2: Node = tree.replace(1, &Node::leaf("y"));
        assert_eq!(new_tree2.to_string(), "(* y c)");
    }

    #[test]
    fn test_display_leaf() {
        let leaf: Node = Node::leaf("variable");
        assert_eq!(leaf.to_string(), "variable");
    }

    #[test]
    fn test_display_branch() {
        let tree: Node = Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]);
        assert_eq!(tree.to_string(), "(+ a b)");
    }

    #[test]
    fn test_display_nested() {
        let tree: Node = Node::branch(
            "*",
            vec![
                Node::branch("+", vec![Node::leaf("a"), Node::leaf("b")]),
                Node::leaf("c"),
            ],
        );
        assert_eq!(tree.to_string(), "(* (+ a b) c)");
    }
}
