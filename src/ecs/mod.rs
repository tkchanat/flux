// use std::{
//   any::{Any, TypeId},
//   collections::{HashMap, HashSet},
// };

// pub trait Component: 'static {}

// pub type EntityId = usize;

// struct JoinIter<'a, J: Join<'a>> {
//   values: Vec<(EntityId, J::Element)>,
// }
// impl<'a, J: Join<'a>> Iterator for JoinIter<'a, J> {
//   type Item = J::Element;
//   fn next(&mut self) -> Option<Self::Item> {
//     if self.values.first().is_none() {
//       None
//     } else {
//       Some(self.values.swap_remove(0).1)
//     }
//   }
// }

// trait Join<'a> {
//   type Storage;
//   type Element;
//   fn join(self) -> JoinIter<'a, Self>
//   where
//     Self: Sized;
// }
// macro_rules! impl_join {
//   ( $($a:ident),+ ) => {
//     impl<'a, $($a: 'a + Joinable<'a>),+> Join<'a> for ($($a,)+)
//     {
//       type Storage = ($($a,)+);
//       type Element = ($($a::Type,)+);
//       #[allow(non_snake_case)]
//       fn join(self) -> JoinIter<'a, Self>
//       where
//         Self: Sized,
//       {
//         let ($($a,)+) = self;
//         let keys = [$($a.keys(),)+];
//         let mut iter = keys.iter();
//         let intersection = iter.next().map(|set| iter.fold(set.to_owned(), |set1, set2| &set1 & set2)).unwrap().to_owned();
//         let values = intersection.iter().map(|id| (*id, ($($a.get(*id).unwrap(),)+))).collect::<Vec<_>>();
//         JoinIter { values }
//       }
//     }
//   };
// }
// impl_join! { A, B }
// impl_join! { A, B, C }

// trait Joinable<'a> {
//   type Type;
//   fn get(&'a mut self, entity: EntityId) -> Option<Self::Type>;
//   fn keys(&self) -> HashSet<EntityId>;
// }
// enum ReadStorage<'a, C: Component> {
//   Vec(&'a VecStorage<C>),
// }
// impl<'a, C: Component> Joinable<'a> for ReadStorage<'a, C> {
//   type Type = &'a C;
//   fn get(&'a mut self, entity: EntityId) -> Option<Self::Type> {
//     match &self {
//       ReadStorage::Vec(vec) => {
//         let idx = *vec.map.get(&entity).unwrap();
//         vec.data.get(idx).unwrap().as_ref()
//       }
//     }
//   }
//   fn keys(&self) -> HashSet<EntityId> {
//     match &self {
//       ReadStorage::Vec(vec) => vec.map.keys().cloned().collect(),
//     }
//   }
// }
// enum WriteStorage<'a, C: Component> {
//   Vec(&'a mut VecStorage<C>),
// }
// impl<'a, C: Component> Joinable<'a> for WriteStorage<'a, C> {
//   type Type = &'a mut C;
//   fn get(&'a mut self, entity: EntityId) -> Option<Self::Type> {
//     match self {
//       WriteStorage::Vec(vec) => {
//         let idx = *vec.map.get(&entity).unwrap();
//         vec.data.get_mut(idx).unwrap().as_mut()
//       }
//     }
//   }
//   fn keys(&self) -> HashSet<EntityId> {
//     match &self {
//       WriteStorage::Vec(vec) => vec.map.keys().cloned().collect(),
//     }
//   }
// }
// trait Storage {
//   fn as_any(&self) -> &dyn Any;
//   fn as_any_mut(&mut self) -> &mut dyn Any;
// }

// #[derive(Default)]
// pub struct World {
//   storages: HashMap<TypeId, Box<dyn Storage>>,
// }
// impl World {
//   fn register<C: Component>(&mut self) {
//     let type_id = TypeId::of::<C>();
//     self
//       .storages
//       .insert(type_id, Box::new(VecStorage::<C>::new()));
//   }
//   fn read_storage<C: Component>(&self) -> ReadStorage<'_, C> {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get(&type_id) {
//       return ReadStorage::Vec(storage.as_any().downcast_ref::<VecStorage<C>>().unwrap());
//     }
//     unreachable!();
//   }
//   fn write_storage<C: Component>(&self) -> WriteStorage<'_, C> {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get_mut(&type_id) {
//       return WriteStorage::Vec(
//         storage
//           .as_any_mut()
//           .downcast_mut::<VecStorage<C>>()
//           .unwrap(),
//       );
//     }
//     unreachable!();
//   }
// }

// #[derive(Default)]
// pub struct VecStorage<C: Component> {
//   data: Vec<Option<C>>,
//   map: HashMap<EntityId, usize>,
// }
// impl<C: Component> VecStorage<C> {
//   fn new() -> Self {
//     Self {
//       data: Vec::new(),
//       map: HashMap::new(),
//     }
//   }
// }
// impl<C: Component> Storage for VecStorage<C> {
//   fn as_any(&self) -> &dyn Any {
//     self as &dyn Any
//   }
//   fn as_any_mut(&mut self) -> &mut dyn Any {
//     self as &mut dyn Any
//   }
// }

// impl Component for i32 {}
// impl Component for f32 {}
// impl Component for String {}
// #[test]
// fn test() {
//   let mut world = World::default();
//   world.register::<i32>();
//   world.register::<f32>();
//   world.register::<String>();

//   let int = world.read_storage::<i32>();
//   let float = world.read_storage::<f32>();
//   let string = world.write_storage::<String>();
//   for (i, f, s) in (int, float, string).join() {
//     println!("i={}, f={}", i, f);
//   }
// }
