#[macro_use]
use std::cell::{Ref, RefCell};
use std::cmp::PartialEq;
use std::rc::{Rc, Weak};

#[typetag::serde]
pub trait Component: 'static {
  fn type_id() -> u32
  where
    Self: Sized,
  {
    const_fnv1a_hash::fnv1a_hash_str_32(Self::type_name())
  }
  fn type_name() -> &'static str
  where
    Self: Sized;
  fn init(&mut self) {}
  fn start(&mut self) {}
  fn update(&mut self, dt: f32) {}
  fn destroy(&mut self) {}
}

// pub trait ComponentType {
//   fn identifier() -> u32;
// }
// #[macro_export]
// macro_rules! impl_component_type {
//   ($name: ident) => {
//     impl ComponentType for $name {
//       fn identifier() -> u32 {
//         const_fnv1a_hash::fnv1a_hash_str_32(stringify!($name))
//       }
//     }
//   };
// }
// pub use impl_component_type;

#[derive(serde::Serialize, serde::Deserialize)]
struct TypedComponent {
  #[serde(skip_serializing, skip_deserializing)]
  ty: u32,
  component: Box<dyn Component>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeData {
  name: String,
  #[serde(skip_serializing, skip_deserializing)]
  parent: Weak<RefCell<NodeData>>,
  children: Vec<Node>,
  components: Vec<TypedComponent>,
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

  pub fn add_child(&self, child: Node) {
    {
      let mut data = self.0.borrow_mut();
      data.children.push(child);
    }
  }

  pub fn set_parent(&self, parent: &Node) {
    let mut data = self.0.borrow_mut();
    if let Some(old_parent) = data.parent.upgrade() {
      if Rc::ptr_eq(&self.0, &old_parent) {
        return; // Assign to the same node as parent
      }

      let children = &mut old_parent.borrow_mut().children;
      if let Some(index) = children.iter().position(|x| x == self) {
        children.remove(index);
      }
    }
    data.parent = Rc::downgrade(&parent.0);
    parent.add_child(Node(self.0.clone()));
  }

  pub fn add_component<C: Component>(&self, component: C) {
    let mut data = self.0.borrow_mut();
    data.components.push(TypedComponent {
      ty: C::type_id(),
      component: Box::new(component),
    });
  }

  pub fn get_component<C: Component>(&self) -> Option<Ref<'_, C>> {
    let data = self.0.borrow();
    data
      .components
      .iter()
      .position(|comp| comp.ty == C::type_id())
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
impl serde::ser::Serialize for Node {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let value = unsafe { &*(&*self.0.as_ptr() as *const NodeData) };
    serializer.serialize_newtype_struct("Node", value)
  }
}
impl<'de> serde::de::Deserialize<'de> for Node {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let mut node_data = NodeData::deserialize(deserializer)?;
    for typed in node_data.components.iter_mut() {
      typed.ty = const_fnv1a_hash::fnv1a_hash_str_32(typed.component.typetag_name());
    }
    let node = Self(Rc::new(RefCell::new(node_data)));
    for child in node.children().iter() {
      let mut data = child.0.borrow_mut();
      data.parent = Rc::downgrade(&node.0);
    }
    Ok(node)
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
  impl Component for Foo {
    fn type_name() -> &'static str {
      "Foo"
    }
  }

  #[test]
  fn test_serde() {
    let foo = Foo(123);
    let ser = serde_json::to_string(&foo).unwrap();
    println!("Serialized foo: {:?}", ser);
    let de: Foo = serde_json::from_str(&ser).unwrap();
    assert!(foo.0 == de.0);

    // Serialize
    let node = Node::new("node");
    let child = Node::new("child");
    node.add_component(foo);
    node.add_child(child);
    let ser = serde_json::to_string(&node).expect("Unable to serialize node");
    println!("Serialized node {:?}", ser);

    // Deserialize
    let node = serde_json::from_str::<Node>(&ser).expect("Unable to deserialize node");
    assert!(node.parent() == None);
    assert!(node.children().len() == 1);
    assert!(node.children()[0].get_component::<Foo>().is_none());
    assert!(node.children()[0].parent().is_some());
    assert!(node.children()[0].parent().unwrap() == node);
    assert!(node.get_component::<Foo>().is_some());
    assert!(node.get_component::<Foo>().unwrap().0 == 123);
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
