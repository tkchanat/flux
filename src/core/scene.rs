use super::app;
use crate::{gfx, prefabs};
use core::any::TypeId;
use specs::{
  Builder, Component, DenseVecStorage, Entity, Join, ReadStorage, System, World, WorldExt,
};
use specs_derive::Component;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
};

fn world() -> &'static specs::World {
  &app().scene.world
}
fn world_mut() -> &'static mut specs::World {
  &mut app().scene.world
}

#[derive(Component, Default)]
struct Relationship {
  parent: Option<Entity>,
  children: Vec<Entity>,
}

pub struct Node {
  pub entity: Entity,
}
impl Node {
  pub fn new() -> Self {
    app().scene.create_node(None)
  }
  pub fn destroy(self) {
    app().scene.destroy_node(self);
  }
  pub fn set_parent(&self, parent: &Node) {
    self.get_component_mut(|rel: &mut Relationship| {
      // Remove relationship from the old parent
      if let Some(old_parent) = rel.parent {
        if let Some(mut old_parent_rel) = world()
          .write_component::<Relationship>()
          .get_mut(old_parent)
        {
          let idx = old_parent_rel
            .children
            .iter()
            .position(|x| *x == self.entity)
            .expect("Entity was never added to parent relationship");
          old_parent_rel.children.remove(idx);
        }
      }
      // Establish new relationship with the new parent
      if let Some(mut new_parent_rel) = world()
        .write_component::<Relationship>()
        .get_mut(parent.entity)
      {
        new_parent_rel.children.push(self.entity);
      }
      rel.parent = Some(parent.entity);
    });
  }
  pub fn get_parent(&self) -> Option<Node> {
    let mut parent = None;
    self.get_component(|rel: &Relationship| {
      if let Some(entity) = rel.parent {
        parent = Some(Node { entity });
      }
    });
    parent
  }
  pub fn children(&self) -> Vec<Node> {
    let mut children = Vec::new();
    self.get_component(|rel: &Relationship| {
      children = rel.children.iter().map(|x| Node { entity: *x }).collect();
    });
    children
  }
  pub fn add_child(&self) -> Node {
    let child = app().scene.create_node(Some(self.entity));
    self.get_component_mut(|rel: &mut Relationship| {
      rel.parent = Some(self.entity);
      rel.children.push(child.entity);
    });
    child
  }
  pub fn add_component<C: Component>(&self, component: C) -> &Node {
    let type_id = TypeId::of::<C>();
    let observers = app().scene.observers.borrow();
    if let Some(observers) = observers.get(&type_id) {
      for obs in observers {
        let cb = &obs.as_any().downcast_ref::<Observer<C>>().unwrap().cb;
        cb(&self, &component);
      }
    }
    world()
      .write_component::<C>()
      .insert(self.entity, component)
      .expect("Unable to add component to entity");
    self
  }
  pub fn remove_component<C: Component>(&self) -> Option<C> {
    world().write_component::<C>().remove(self.entity)
  }
  pub fn get_component<C: Component, F: FnMut(&C)>(&self, mut f: F) {
    let storage = world().read_component::<C>();
    if let Some(component) = storage.get(self.entity) {
      f(component);
    }
  }
  pub fn get_component_mut<C: Component, F: FnMut(&mut C)>(&self, mut f: F) {
    let mut storage = world().write_component::<C>();
    if let Some(component) = storage.get_mut(self.entity) {
      f(component);
    }
  }
}
trait AsAny {
  fn as_any(&self) -> &dyn std::any::Any;
}
struct Observer<C: Component> {
  cb: Box<dyn Fn(&Node, &C)>,
}
impl<C: Component> AsAny for Observer<C> {
  fn as_any(&self) -> &dyn std::any::Any {
    self as &dyn std::any::Any
  }
}
pub struct Scene {
  pub(super) world: specs::World,
  pending_kill: VecDeque<Entity>,
  observers: RefCell<HashMap<TypeId, Vec<Box<dyn AsAny>>>>,
  pub root: Node,
}
impl Scene {
  pub fn observe<C: Component, F: Fn(&Node, &C) + 'static>(&self, f: F) {
    let type_id = TypeId::of::<C>();
    let cb = Box::new(Observer::<C> { cb: Box::new(f) });
    let mut observers = self.observers.borrow_mut();
    match observers.get_mut(&type_id) {
      Some(observers) => observers.push(cb),
      None => {
        observers.insert(type_id, vec![cb]);
      }
    }
  }
  pub fn new() -> Self {
    let mut world = World::new();
    world.register::<Relationship>();
    world.register::<prefabs::GeomSphere>();
    world.register::<prefabs::Camera>();
    world.register::<gfx::Transform>();
    world.register::<gfx::Mesh>();
    world.register::<gfx::MaterialOverlay>();

    let root = Node {
      entity: world
        .create_entity()
        .with::<Relationship>(Relationship::default())
        .build(),
    };
    Self {
      world,
      root,
      observers: RefCell::new(HashMap::new()),
      pending_kill: VecDeque::new(),
    }
  }
  fn create_node(&mut self, parent: Option<Entity>) -> Node {
    if parent.is_none() {
      return self.root.add_child();
    }
    Node {
      entity: self
        .world
        .create_entity()
        .with::<Relationship>(Relationship {
          parent,
          children: Vec::new(),
        })
        .build(),
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
        .delete_entity(entity)
        .expect("Unable to delete entity");
    }
  }
  pub fn each<T: Component, F: FnMut(&T)>(&self, mut f: F) {
    let storage = self.world.read_storage::<T>();
    for s in (&storage).join() {
      f(s);
    }
  }
}
impl Drop for Scene {
  fn drop(&mut self) {
    self
      .world
      .delete_entity(self.root.entity)
      .expect("Unable to delete entity");
  }
}
