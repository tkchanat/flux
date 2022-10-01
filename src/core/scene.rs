use crate::app;
use log::warn;
use owning_ref::{OwningRef, RwLockReadGuardRef};
use specs::{Builder, Component, DenseVecStorage, Entity, ReadStorage, World, WorldExt, Write};
use specs_derive::Component;
use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
  sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak},
};

#[derive(Component, Default)]
struct Relationship {
  parent: Option<Entity>,
  children: Vec<Entity>,
}

pub struct NodeRef {
  entity: Entity,
}

pub struct ReadComponent<C: Component>(*const C);
impl<C: Component> Deref for ReadComponent<C> {
  type Target = C;
  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref().unwrap() }
  }
}
impl<C: Component> Deref for WriteComponent<C> {
  type Target = C;
  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref().unwrap() }
  }
}
pub struct WriteComponent<C: Component>(*mut C);
impl<C: Component> DerefMut for WriteComponent<C> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut().unwrap() }
  }
}

trait ComponentsTuple {
  type Type;
  fn compose(self) -> Self::Type;
}

pub struct Node {
  entity: Entity,
  world: Weak<RwLock<World>>,
}
impl Node {
  pub fn new() -> Self {
    app().scene.create_node()
  }
  pub fn destroy(self) {
    app().scene.destroy_node(self);
  }
  pub fn set_parent(&self, parent: &Node) {
    // self.get_component_mut::<Relationship>();
  }
  pub fn add_component<C: Component>(&self, component: C) {
    self
      .world()
      .write()
      .unwrap()
      .write_component::<C>()
      .insert(self.entity, component)
      .expect("Unable to add component to entity");
  }
  pub fn remove_component<C: Component>(&self) -> Option<C> {
    self
      .world()
      .write()
      .unwrap()
      .write_component::<C>()
      .remove(self.entity)
  }
  pub fn get_component<T>(&self) -> Option<T> {
    let world = self.world().read().unwrap();
    world.write_component()
  }
  fn world(&self) -> Arc<RwLock<World>> {
    self.world.upgrade().expect("World no longer exists")
  }
}

pub struct Scene {
  world: Arc<RwLock<World>>,
  root: Entity,
  pending_kill: VecDeque<Entity>,
}
impl Scene {
  pub fn new() -> Self {
    let mut world = World::new();
    world.register::<Relationship>();
    let root = world
      .create_entity()
      .with::<Relationship>(Relationship::default())
      .build();
    Self {
      world: Arc::new(RwLock::new(world)),
      root,
      pending_kill: VecDeque::new(),
    }
  }
  pub fn create_node(&self) -> Node {
    Node {
      entity: self
        .world
        .write()
        .unwrap()
        .create_entity()
        .with::<Relationship>(Relationship {
          parent: Some(self.root),
          children: Vec::new(),
        })
        .build(),
      world: Arc::downgrade(&self.world),
    }
  }
  fn destroy_node(&mut self, node: Node) {
    self.pending_kill.push_back(node.entity);
  }
  pub fn update(&mut self) {
    while !self.pending_kill.is_empty() {
      let entity = self.pending_kill.pop_back().unwrap();
      self
        .world
        .write()
        .unwrap()
        .delete_entity(entity)
        .expect("Unable to delete entity");
    }
  }
}
impl Drop for Scene {
  fn drop(&mut self) {
    self
      .world
      .write()
      .unwrap()
      .delete_entity(self.root)
      .expect("Unable to delete entity");
  }
}
