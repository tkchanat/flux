use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use serde::ser::SerializeStruct;

#[typetag::serde]
pub trait Component: 'static {
  fn init(&mut self) {}
  fn start(&mut self) {}
  fn update(&mut self, dt: f32) {}
  fn destroy(&mut self) {}
}

pub trait ComponentType {
  fn name() -> &'static str;
  fn identifier() -> u32;
}
macro_rules! impl_component_type {
  ($name: ident) => {
    impl ComponentType for $name {
      fn name() -> &'static str {
        stringify!($name)
      }
      fn identifier() -> u32 {
        const_fnv1a_hash::fnv1a_hash_str_32(stringify!($name))
      }
    }
  };
}

struct TypedComponent {
  ty: u32,
  component: Box<dyn Component>,
}
impl serde::ser::Serialize for TypedComponent {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.component.serialize(serializer)
  }
}

struct NodeData {
  name: String,
  parent: Weak<RefCell<NodeData>>,
  children: Vec<Node>,
  components: Vec<TypedComponent>,
}
impl serde::ser::Serialize for NodeData {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut state = serializer.serialize_struct("NodeData", std::mem::size_of::<NodeData>())?;
    state.serialize_field("name", &self.name)?;
    state.serialize_field("children", &self.children)?;
    state.serialize_field("components", &self.components)?;
    state.end()
  }
}

pub struct Node(Rc<RefCell<NodeData>>);
impl serde::ser::Serialize for Node {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let value = unsafe { &*(&*self.0.as_ptr() as *const NodeData) };
    serializer.serialize_newtype_struct("Node", value)
  }
}
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

  pub fn add_component<C: Component + ComponentType>(&self, component: C) {
    let mut data = self.0.borrow_mut();
    data.components.push(TypedComponent {
      ty: C::identifier(),
      component: Box::new(component),
    });
  }

  pub fn get_component<C: ComponentType>(&self) -> Option<Ref<'_, C>> {
    let data = self.0.borrow();
    data
      .components
      .iter()
      .position(|comp| comp.ty == C::identifier())
      .and_then(|index| {
        Some(Ref::map(data, |data| unsafe {
          &*(std::ptr::addr_of!(*(&data.components[index]).component.as_ref()).cast::<C>())
        }))
      })
  }
}
impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

#[derive(Clone)]
pub struct NodeRef(Weak<RefCell<NodeData>>);

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(serde::Serialize, serde::Deserialize)]
  struct Foo(u32);

  #[typetag::serde]
  impl Component for Foo {}
  impl_component_type!(Foo);

  #[test]
  fn test_serialization() {
    let foo = Foo(123);
    let ser = serde_json::to_string(&foo).unwrap();
    println!("Serialized foo: {:?}", ser);
    let de: Foo = serde_json::from_str(&ser).unwrap();
    assert!(foo.0 == de.0);

    let node = Node::new("node");
    let child = Node::new("child");
    node.add_component(foo);
    node.add_child(child);
    let ser = serde_json::to_string(&node).unwrap();
    println!("Serialized node {:?}", ser);
  }

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

  #[test]
  fn test_components() {
    let node = Node::new("node");
    node.add_component(Foo(123));

    let comp = node.get_component::<Foo>();
    assert!(comp.is_some());
    assert!(comp.unwrap().0 == 123);
  }
}
