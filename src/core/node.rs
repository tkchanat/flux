use std::cell::{Ref, RefCell, RefMut};
use std::cmp::PartialEq;
use std::rc::{Rc, Weak};

pub trait Component {
  fn init(&mut self) {}
  fn start(&mut self) {}
  fn update(&mut self, dt: f32) {}
  fn destroy(&mut self) {}
}

struct NodeData {
  name: String,
  parent: Weak<RefCell<NodeData>>,
  children: Vec<Node>,
  components: Vec<Box<dyn Component>>,
}

pub struct Node(Rc<RefCell<NodeData>>);
impl Node {
  pub fn new(name: &str) -> Self {
    Self(Rc::new(RefCell::new(NodeData {
      name: String::from(name),
      parent: Weak::new(),
      children: Vec::new(),
      components: Vec::new(),
    })))
  }

  pub fn name(&self) -> Ref<'_, String> {
    Ref::map(self.0.borrow(), |data| &data.name)
  }

  pub fn children(&self) -> Ref<'_, [Node]> {
    Ref::map(self.0.borrow(), |this| this.children.as_slice())
  }

  pub fn parent(&self) -> Option<Node> {
    self
      .0
      .borrow()
      .parent
      .upgrade()
      .and_then(|parent| Some(Node(parent)))
  }

  pub fn add_child(&self, child: Node) -> Ref<'_, Node> {
    {
      let mut data = self.0.borrow_mut();
      data.children.push(child);
    }
    Ref::map(self.0.borrow(), |node| node.children.last().unwrap())
  }

  pub fn set_parent(&self, parent: &Node) {
    if let Some(old_parent) = self.0.borrow().parent.upgrade() {
      let children = &mut old_parent.borrow_mut().children;
      if let Some(index) = children.iter().position(|x| x == self) {
        children.remove(index);
      }
    }
    self.0.borrow_mut().parent = Rc::downgrade(&parent.0);
    parent.add_child(Node(self.0.clone()));
  }
}
impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

#[derive(Clone)]
pub struct NodeRef(Weak<RefCell<NodeData>>);

#[test]
fn test_relationship() {
  let node = Node::new("node");
  assert!(node.parent() == None);

  let parent = Node::new("parent");
  node.set_parent(&parent);
  assert!(node.parent().unwrap() == parent);
  assert_eq!(parent.children().len(), 1);

  let new_parent = Node::new("new_parent");
  node.set_parent(&new_parent);
  assert!(node.parent().unwrap() == new_parent);
  assert_eq!(parent.children().len(), 0);
  assert_eq!(new_parent.children().len(), 1);
}
