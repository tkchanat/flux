use std::sync::{Arc, RwLock, Weak};

use log::warn;

// use crate::ecs::{world::World, Entity};

// pub struct Node {
//   entity: Entity,
//   world: Weak<RwLock<World>>,
// }
// impl Drop for Node {
//   fn drop(&mut self) {
//     match self.world.upgrade() {
//       Some(world) => world.write().unwrap().destroy_entity(&self.entity),
//       None => warn!("World no longer exists!"),
//     }
//   }
// }

// pub struct Scene {
//   world: Arc<RwLock<World>>,
// }
// impl Scene {
//   pub fn new() -> Self {
//     Self {
//       world: Arc::new(RwLock::new(World::new())),
//     }
//   }

//   pub fn create_node(&self) -> Node {
//     Node {
//       entity: self.world.write().unwrap().create_entity(),
//       world: Arc::downgrade(&self.world),
//     }
//   }
// }
