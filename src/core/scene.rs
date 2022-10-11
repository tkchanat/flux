use crate::{gfx, prefabs};
use super::app;
use core::any::TypeId;
use specs::{
  Builder, Component, DenseVecStorage, DispatcherBuilder, Entity, Join, ReadStorage, System, World,
  WorldExt,
};
use specs_derive::Component;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
};

#[derive(Component, Default)]
struct Relationship {
  parent: Option<Entity>,
  children: Vec<Entity>,
}

pub struct Read<C: Component>(*const C);
impl<C: Component> Deref for Read<C> {
  type Target = C;
  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref().unwrap() }
  }
}
impl<C: Component> Composable for Read<C> {
  fn get(world: &RefMut<World>, entity: Entity) -> Self {
    let ptr = match world.read_component::<C>().get(entity) {
      Some(component) => component as *const C,
      None => std::ptr::null(),
    };
    Read(ptr)
  }
  fn valid(&self) -> bool {
    !self.0.is_null()
  }
}
pub struct Write<C: Component>(*mut C);
impl<C: Component> Deref for Write<C> {
  type Target = C;
  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref().unwrap() }
  }
}
impl<C: Component> DerefMut for Write<C> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut().unwrap() }
  }
}
impl<C: Component> Composable for Write<C> {
  fn get(world: &RefMut<World>, entity: Entity) -> Self {
    let ptr = match world.write_component::<C>().get_mut(entity) {
      Some(component) => component as *mut C,
      None => std::ptr::null_mut(),
    };
    Write(ptr)
  }
  fn valid(&self) -> bool {
    !self.0.is_null()
  }
}

pub trait ComponentsTuple {
  fn compose(world: &RefMut<World>, entity: Entity) -> Option<Self>
  where
    Self: Sized;
}
impl<A: Composable> ComponentsTuple for A {
  fn compose(world: &RefMut<World>, entity: Entity) -> Option<Self> {
    let a = A::get(world, entity);
    if A::valid(&a) {
      Some(a)
    } else {
      None
    }
  }
}
macro_rules! impl_compose {
  ($($a:ident)+) => {
    impl<$($a: Composable),+> ComponentsTuple for ($($a,)+) {
      #[allow(non_snake_case)]
      fn compose(world: &RefMut<World>, entity: Entity) -> Option<Self> {
        let tuple = ($($a::get(world, entity),)+);
        let ($($a,)+) = tuple;
        // let (a, b) = (A::get(world, entity), B::get(world, entity));
        let all_valid = [$($a.valid(),)+].iter().all(|x| *x);
        if all_valid {
          Some(($($a,)+))
        } else {
          None
        }
      }
    }
  };
}
impl_compose! { A }
impl_compose! { A B }
impl_compose! { A B C }
impl_compose! { A B C D }
impl_compose! { A B C D E }
impl_compose! { A B C D E F }
impl_compose! { A B C D E F G }
impl_compose! { A B C D E F G H }

trait Composable {
  fn get(world: &RefMut<World>, entity: Entity) -> Self;
  fn valid(&self) -> bool;
}

pub struct Node {
  entity: Entity,
  world: Weak<RefCell<specs::World>>,
}
impl Node {
  pub fn new() -> Self {
    app().scene.create_node(None)
  }
  pub fn destroy(self) {
    app().scene.destroy_node(self);
  }
  pub fn set_parent(&self, parent: &Node) {
    if let Some(mut rel) = self.get_component::<Write<Relationship>>() {
      // Remove relationship from the old parent
      if let Some(old_parent) = rel.parent {
        if let Some(mut old_parent_rel) = self.get_component_impl::<Write<Relationship>>(old_parent)
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
      if let Some(mut new_parent_rel) =
        self.get_component_impl::<Write<Relationship>>(parent.entity)
      {
        new_parent_rel.children.push(self.entity);
      }
      rel.parent = Some(parent.entity);
    }
  }
  pub fn get_parent(&self) -> Option<Node> {
    if let Some(rel) = self.get_component::<Read<Relationship>>() {
      if let Some(parent) = rel.parent {
        return Some(Node {
          entity: parent,
          world: self.world.clone(),
        });
      }
    }
    None
  }
  pub fn children(&self) -> Vec<Node> {
    if let Some(rel) = self.get_component::<Read<Relationship>>() {
      return rel
        .children
        .iter()
        .map(|x| Node {
          entity: *x,
          world: self.world.clone(),
        })
        .collect();
    }
    Vec::new()
  }
  pub fn add_child(&self) -> Node {
    let child = app().scene.create_node(Some(self.entity));
    if let Some(mut rel) = self.get_component::<Write<Relationship>>() {
      rel.parent = Some(self.entity);
      rel.children.push(child.entity);
    }
    child
  }
  pub fn add_component<C: Component>(&self, component: C) {
    let type_id = TypeId::of::<C>();
    let observers = app().scene.observers.borrow();
    if let Some(observers) = observers.get(&type_id) {
      for obs in observers {
        let cb = &obs.as_any().downcast_ref::<Observer<C>>().unwrap().cb;
        cb(&self, &component);
      }
    }
    self
      .world
      .upgrade()
      .expect("World no longer exists")
      .borrow_mut()
      .write_component::<C>()
      .insert(self.entity, component)
      .expect("Unable to add component to entity");
  }
  pub fn remove_component<C: Component>(&self) -> Option<C> {
    self
      .world
      .upgrade()
      .expect("World no longer exists")
      .borrow_mut()
      .write_component::<C>()
      .remove(self.entity)
  }
  pub fn get_component<T: ComponentsTuple>(&self) -> Option<T> {
    self.get_component_impl(self.entity)
  }
  fn get_component_impl<T: ComponentsTuple>(&self, entity: Entity) -> Option<T> {
    let world = self.world.upgrade().expect("World no longer exists");
    let x = T::compose(&world.borrow_mut(), entity);
    x
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
  world: Rc<RefCell<specs::World>>,
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

    let handle = Rc::new(RefCell::new(world));
    let root = Node {
      entity: handle
        .borrow_mut()
        .create_entity()
        .with::<Relationship>(Relationship::default())
        .build(),
      world: Rc::downgrade(&handle),
    };
    Self {
      world: handle,
      root,
      observers: RefCell::new(HashMap::new()),
      pending_kill: VecDeque::new(),
    }
  }
  fn create_node(&self, parent: Option<Entity>) -> Node {
    if parent.is_none() {
      return self.root.add_child();
    }
    Node {
      entity: self
        .world
        .borrow_mut()
        .create_entity()
        .with::<Relationship>(Relationship {
          parent,
          children: Vec::new(),
        })
        .build(),
      world: Rc::downgrade(&self.world),
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
        .borrow_mut()
        .delete_entity(entity)
        .expect("Unable to delete entity");
    }
  }
  // pub fn storage<'a, C: Component>(&'a self) -> specs::ReadStorage<'a, C> {
  //   self.world.borrow().read_storage::<C>()
  // }
  // pub fn storage_mut<'a, C: Component>(&'a self) -> specs::WriteStorage<'a, C> {
  //   self.world.borrow().write_storage::<C>()
  // }
  pub fn world(&self) -> Ref<World> {
    self.world.borrow()
  }
  pub fn each<T: Component, F: FnMut(&T)>(&self, mut f: F) {
    let world = self.world.borrow();
    let storage = world.read_storage::<T>();
    for s in (&storage).join() {
      f(s);
    }
  }
}
impl Drop for Scene {
  fn drop(&mut self) {
    self
      .world
      .borrow_mut()
      .delete_entity(self.root.entity)
      .expect("Unable to delete entity");
  }
}
