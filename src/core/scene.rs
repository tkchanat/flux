use super::node::Node;

pub struct Scene {
  pub root: Node,
}
impl Scene {
  pub fn new() -> Self {
    Self {
      root: Node::new("root"),
    }
  }
}
