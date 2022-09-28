use std::{
  any::{Any, TypeId},
  collections::{HashMap, HashSet},
  vec::IntoIter,
};

pub trait Component: 'static {}

pub type EntityId = usize;

trait JoinIter<'a> {
  
}
trait Join<'a> {
  type Storage;
  type Element;
  fn join(self) -> IntoIter<Self::Element>
  where
    Self: Sized;
}
macro_rules! impl_join {
  ( $($a:ident),+ ) => {
    impl<'a, $($a: Joinable<'a>),+> Join<'a> for ($($a,)+)
    {
      type Storage = ($($a,)+);
      type Element = ($($a::Type,)+);
      #[allow(non_snake_case)]
      fn join(self) -> IntoIter<Self::Element>
      where
        Self: Sized,
      {
        let ($(mut $a,)+) = self;
        let keys = [$($a.keys(),)+];
        let mut iter = keys.iter();
        let intersection = iter.next().map(|set| iter.fold(set.to_owned(), |set1, set2| &set1 & set2)).unwrap().to_owned();
        intersection.iter().map(|id| ($($a.get(*id).unwrap(),)+)).collect::<Vec<_>>().into_iter()
      }
    }
  };
}
impl_join! { A, B }
impl_join! { A, B, C }

// impl<'a, 'b: 'a, A: Joinable<'a>, B: Joinable<'a>> Join<'a> for (A, B) {
//   type Storage = (A, B);
//   type Element = (A::Type, B::Type);

//   fn join(mut self) -> IntoIter<Self::Element>
//   where
//     Self: Sized,
//   {
//     let (a, b) = self;
//     let keys = [a.keys(), b.keys()];
//     let iter = keys.iter();
//     let intersection = iter
//       .next()
//       .map(|set| iter.fold(set, |set1, set2| &(set1 & set2)))
//       .unwrap()
//       .to_owned();
//     intersection
//       .iter()
//       .map(|id| (a.get(*id).unwrap(), b.get(*id).unwrap()))
//       .collect::<Vec<_>>()
//       .into_iter()
//   }
// }

trait Joinable<'a> {
  type Type;
  fn get(&'a mut self, entity: EntityId) -> Option<Self::Type>;
  fn keys(&self) -> HashSet<EntityId>;
}
enum ReadStorage<'a, C: Component> {
  Vec(&'a VecStorage<C>),
}
impl<'a, C: Component> Joinable<'a> for ReadStorage<'a, C> {
  type Type = &'a C;
  fn get(&'a mut self, entity: EntityId) -> Option<Self::Type> {
    match &self {
      ReadStorage::Vec(vec) => {
        let idx = *vec.map.get(&entity).unwrap();
        vec.data.get(idx).unwrap().as_ref()
      }
    }
  }
  fn keys(&self) -> HashSet<EntityId> {
    match &self {
      ReadStorage::Vec(vec) => vec.map.keys().cloned().collect(),
    }
  }
}
enum WriteStorage<'a, C: Component> {
  Vec(&'a mut VecStorage<C>),
}
// impl<'a, C: Component> WriteStorage<'a, C> {
//   fn get(&'a mut self, entity: EntityId) -> Option<&'a mut C> {
//     match &self {
//       WriteStorage::Vec(vec) => {
//         let idx = *vec.map.get(&entity).unwrap();
//         vec.data.get_mut(idx).unwrap().as_mut()
//       }
//     }
//   }
// }
impl<'a, C: Component> Joinable<'a> for WriteStorage<'a, C> {
  type Type = &'a mut C;
  fn get(&'a mut self, entity: EntityId) -> Option<Self::Type> {
    match self {
      WriteStorage::Vec(vec) => {
        let idx = *vec.map.get(&entity).unwrap();
        vec.data.get_mut(idx).unwrap().as_mut()
      }
    }
  }
  fn keys(&self) -> HashSet<EntityId> {
    match &self {
      WriteStorage::Vec(vec) => vec.map.keys().cloned().collect(),
    }
  }
}
trait Storage {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub struct World {
  storages: HashMap<TypeId, Box<dyn Storage>>,
}
impl World {
  fn register<C: Component>(&mut self) {
    let type_id = TypeId::of::<C>();
    self
      .storages
      .insert(type_id, Box::new(VecStorage::<C>::new()));
  }
  fn read_storage<C: Component>(&self) -> ReadStorage<'_, C> {
    let type_id = TypeId::of::<C>();
    if let Some(storage) = self.storages.get(&type_id) {
      return ReadStorage::Vec(storage.as_any().downcast_ref::<VecStorage<C>>().unwrap());
    }
    unreachable!();
  }
  fn write_storage<C: Component>(&mut self) -> WriteStorage<'_, C> {
    let type_id = TypeId::of::<C>();
    if let Some(storage) = self.storages.get_mut(&type_id) {
      return WriteStorage::Vec(
        storage
          .as_any_mut()
          .downcast_mut::<VecStorage<C>>()
          .unwrap(),
      );
    }
    unreachable!();
  }
}

#[derive(Default)]
pub struct VecStorage<C: Component> {
  data: Vec<Option<C>>,
  map: HashMap<EntityId, usize>,
}
impl<C: Component> VecStorage<C> {
  fn new() -> Self {
    Self {
      data: Vec::new(),
      map: HashMap::new(),
    }
  }
}
impl<C: Component> Storage for VecStorage<C> {
  fn as_any(&self) -> &dyn Any {
    self as &dyn Any
  }
  fn as_any_mut(&mut self) -> &mut dyn Any {
    self as &mut dyn Any
  }
}

impl Component for i32 {}
impl Component for f32 {}
impl Component for String {}
#[test]
fn test() {
  let mut world = World::default();
  world.register::<i32>();
  world.register::<f32>();
  world.register::<String>();

  let int = world.read_storage::<i32>();
  let float = world.read_storage::<f32>();
  let string = world.read_storage::<String>();
  for (i, f, s) in (int, float, string).join() {
    println!("i={}, f={}", i, f);
  }
}

// macro_rules! define_open {
//   // use variables to indicate the arity of the tuple
//   ($($a:ty),*) => {

//   }
// }
// define_open! {A}
// define_open! {A, B}
// define_open! {A, B, C}
// define_open! {A, B, C, D}
// define_open! {A, B, C, D, E}
// define_open! {A, B, C, D, E, F}
// define_open! {A, B, C, D, E, F, G}
// define_open! {A, B, C, D, E, F, G, H}

// pub trait Storage {
//   fn as_any(&self) -> &dyn Any;
//   fn as_any_mut(&mut self) -> &mut dyn Any;
//   fn node_destroyed(&mut self, entity: &Entity);
// }
// pub struct VecStorage<C: Component> {
//   components: Vec<C>,
//   node_component_map: bimap::BiMap<usize, usize>,
// }
// impl<C: Component> VecStorage<C> {
//   pub(super) fn new() -> Self {
//     Self {
//       components: Vec::new(),
//       node_component_map: bimap::BiMap::new(),
//     }
//   }
//   pub(super) fn insert_data(&mut self, entity: &Entity, component: C) {
//     assert!(
//       self.node_component_map.get_by_left(&entity.id).is_none(),
//       "Component already added"
//     );
//     let comp_idx = self.components.len();
//     self.node_component_map.insert(entity.id, comp_idx);
//     if comp_idx >= self.components.capacity() {
//       self.components.reserve(self.components.capacity());
//     }
//     self.components.push(component);
//   }
//   pub(super) fn remove_data(&mut self, entity: &Entity) {
//     assert!(
//       self.node_component_map.get_by_left(&entity.id).is_some(),
//       "Node not registered"
//     );
//     let removed_idx = *self.node_component_map.get_by_left(&entity.id).unwrap();
//     let last_idx = self.components.len() - 1;
//     self.components.swap(removed_idx, last_idx);
//     let last_node_idx = *self.node_component_map.get_by_right(&last_idx).unwrap();
//     self.node_component_map.insert(last_node_idx, last_idx);
//     self.node_component_map.remove_by_left(&entity.id);
//   }
//   pub(super) fn get_data(&self, entity: &Entity) -> Option<&C> {
//     if let Some(comp_idx) = self.node_component_map.get_by_left(&entity.id) {
//       Some(&self.components[*comp_idx]);
//     }
//     None
//   }
//   pub(super) fn get_data_mut(&mut self, entity: &Entity) -> Option<&mut C> {
//     if let Some(comp_idx) = self.node_component_map.get_by_left(&entity.id) {
//       Some(&mut self.components[*comp_idx]);
//     }
//     None
//   }

//   pub(super) fn iter(&self) -> Iter<C> {
//     self.components.iter()
//   }
// }
// impl<C: Component> Storage for VecStorage<C> {
//   fn as_any(&self) -> &dyn Any {
//     self as &dyn Any
//   }
//   fn as_any_mut(&mut self) -> &mut dyn Any {
//     self as &mut dyn Any
//   }
//   fn node_destroyed(&mut self, entity: &Entity) {
//     if self.node_component_map.get_by_left(&entity.id).is_some() {
//       self.remove_data(entity);
//     }
//   }
// }
