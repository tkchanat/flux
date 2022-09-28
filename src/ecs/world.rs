// use super::{
//   storage::{Component, Storage, VecStorage},
//   Entity,
// };
// use std::any::TypeId;
// use std::collections::{HashMap, VecDeque};

// const MAX_ENTITIES: usize = 65536;

// pub struct World {
//   storages: HashMap<TypeId, Box<dyn Storage>>,
//   entities: VecDeque<Entity>,
// }
// impl World {
//   pub fn new() -> Self {
//     Self {
//       storages: HashMap::new(),
//       entities: VecDeque::from_iter((0..MAX_ENTITIES).map(|id| Entity { id, gen: 0 })),
//     }
//   }

//   pub fn create_entity(&mut self) -> Entity {
//     assert!(!self.entities.is_empty(), "No more entities to give!");
//     let entity = self.entities.pop_front().unwrap();
//     entity
//   }

//   pub fn destroy_entity(&mut self, entity: &Entity) {
//     assert!(entity.id < MAX_ENTITIES, "Entity out of range.");
//     for storage in &mut self.storages {
//       storage.1.as_mut().node_destroyed(entity);
//     }
//     self.entities.push_back(Entity {
//       id: entity.id,
//       gen: entity.gen + 1,
//     });
//   }

//   fn register<C: Component>(&mut self) {
//     let type_id = TypeId::of::<C>();
//     assert!(
//       !self.storages.contains_key(&type_id),
//       "Component already registered!"
//     );
//     let storage = VecStorage::<C>::new();
//     self.storages.insert(type_id, Box::new(storage));
//   }

//   pub fn add_component<C: Component>(&mut self, entity: &Entity, component: C) {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get_mut(&type_id) {
//       if let Some(vec_storage) = storage.as_any_mut().downcast_mut::<VecStorage<C>>() {
//         vec_storage.insert_data(entity, component);
//       }
//     } else {
//       self.register::<C>();
//       self.add_component(entity, component);
//     }
//   }

//   pub fn remove_component<C: Component>(&mut self, entity: &Entity) {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get_mut(&type_id) {
//       if let Some(vec_storage) = storage.as_any_mut().downcast_mut::<VecStorage<C>>() {
//         vec_storage.remove_data(entity);
//       }
//     }
//   }

//   pub fn get_component<C: Component>(&self, entity: &Entity) -> Option<&C> {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get(&type_id) {
//       if let Some(vec_storage) = storage.as_any().downcast_ref::<VecStorage<C>>() {
//         return vec_storage.get_data(entity);
//       }
//     }
//     None
//   }

//   pub fn get_component_mut<C: Component>(&mut self, entity: &Entity) -> Option<&mut C> {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get_mut(&type_id) {
//       if let Some(vec_storage) = storage.as_any_mut().downcast_mut::<VecStorage<C>>() {
//         return vec_storage.get_data_mut(entity);
//       }
//     }
//     None
//   }

//   pub fn each<F, C: Component>(&self, cb: F)
//   where
//     F: Fn(&C),
//   {
//     let type_id = TypeId::of::<C>();
//     if let Some(storage) = self.storages.get(&type_id) {
//       if let Some(vec_storage) = storage.as_any().downcast_ref::<VecStorage<C>>() {
//         for comp in vec_storage.iter() {
//           cb(comp);
//         }
//       }
//     }
//   }
// }
