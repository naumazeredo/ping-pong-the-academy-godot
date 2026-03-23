use godot::prelude::*;

pub struct NodeIter {
    root: Gd<Node>,
    current_node: Gd<Node>,
    has_ended: bool,
}

impl NodeIter {
    pub fn new(root: Gd<Node>) -> Self {
        let current_node = root.clone();
        Self {
            root,
            current_node,
            has_ended: false,
        }
    }
}

impl Iterator for NodeIter {
    type Item = Gd<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_ended {
            return None;
        }

        let ret = self.current_node.clone();

        // If has children, go to the first
        if self.current_node.get_child_count() > 0 {
            let Some(child) = self.current_node.get_child(0) else {
                unreachable!()
            };

            self.current_node = child;
        } else {
            // If we are in the root, end the iteration
            if self.current_node == self.root {
                self.has_ended = true;
            } else {
                // Try to find the next sibling
                loop {
                    if let Some(parent) = self.current_node.get_parent() {
                        let index = self.current_node.get_index();

                        if parent.get_child_count() > index + 1 {
                            // If there's a next sibling, go to it

                            let Some(sibling) = parent.get_child(index + 1) else {
                                unreachable!()
                            };

                            self.current_node = sibling;
                            break;
                        } else {
                            // Otherwise, we should go back to the parent and find its next sibling

                            // If we went back to the root, end the iteration
                            if parent == self.root {
                                self.has_ended = true;
                                break;
                            }

                            // Otherwise, go to parent and try the sibling again
                            self.current_node = parent;
                        }
                    } else {
                        // If there's no parent left, end the iteration
                        self.has_ended = true;
                        break;
                    }
                }
            }
        }

        return Some(ret);
    }
}
