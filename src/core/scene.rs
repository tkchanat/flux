use crate::{
  app,
  gfx::Transform,
  prefabs::{Camera, GeomSphere, Mesh},
};
use core::any::TypeId;
use specs::{
  Builder, Component, DenseVecStorage, DispatcherBuilder, Entity, Join, ReadStorage, System, World,
  WorldExt,
};
use specs_derive::Component;
use std::collections::HashMap;
use std::{
  collections::VecDeque,
  ops::{Deref, DerefMut},
  sync::{Arc, RwLock, RwLockReadGuard, Weak},
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
  fn get(world: &RwLockReadGuard<World>, entity: Entity) -> Self {
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
  fn get(world: &RwLockReadGuard<World>, entity: Entity) -> Self {
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
  fn compose(world: &RwLockReadGuard<World>, entity: Entity) -> Option<Self>
  where
    Self: Sized;
}
impl<A: Composable> ComponentsTuple for A {
  fn compose(world: &RwLockReadGuard<World>, entity: Entity) -> Option<Self> {
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
      fn compose(world: &RwLockReadGuard<World>, entity: Entity) -> Option<Self> {
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
  fn get(world: &RwLockReadGuard<World>, entity: Entity) -> Self;
  fn valid(&self) -> bool;
}

pub struct Node {
  entity: Entity,
  world: Weak<RwLock<World>>,
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
    if let Some(observers) = app().scene.observers.get(&type_id) {
      for obs in observers {
        let cb = &obs.as_any().downcast_ref::<Observer<C>>().unwrap().cb;
        cb(&self, &component);
      }
    }
    self
      .world
      .upgrade()
      .expect("World no longer exists")
      .write()
      .unwrap()
      .write_component::<C>()
      .insert(self.entity, component)
      .expect("Unable to add component to entity");
  }
  pub fn remove_component<C: Component>(&self) -> Option<C> {
    self
      .world
      .upgrade()
      .expect("World no longer exists")
      .write()
      .unwrap()
      .write_component::<C>()
      .remove(self.entity)
  }
  pub fn get_component<T: ComponentsTuple>(&self) -> Option<T> {
    self.get_component_impl(self.entity)
  }
  fn get_component_impl<T: ComponentsTuple>(&self, entity: Entity) -> Option<T> {
    let world = self.world.upgrade().expect("World no longer exists");
    let rwlock = world.read().unwrap();
    T::compose(&rwlock, entity)
  }
}
pub trait Depends {
  fn on_added<C: Component>(node: &Node, component: &C)
  where
    Self: Sized;
}
// #[derive(Component)]
// struct Test(i32);
// impl Depends for Test {
//   fn on_added<C: Component>(node: &Node, component: &C)
//   where
//     Self: Sized,
//   {
//     todo!()
//   }
// }
// impl<C: Component> Depends for fn(&Node, &C) {
//   fn on_added<C: Component>(node: &Node, component: &C)
//   where
//     Self: Sized,
//   {
//     todo!()
//   }
// }
// #[test]
// fn test() {
//   let mut scene = Scene::new();
//   let node = scene.create_node(None);
//   node.add_component(Test(1));
//   scene.observe::<Test, _>(|node: &Node, comp: &Test| {
//     println!("id = {:?}", comp.0);
//   });
// }
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
  world: Arc<RwLock<World>>,
  pending_kill: VecDeque<Entity>,
  observers: HashMap<TypeId, Vec<Box<dyn AsAny>>>,
  pub root: Node,
}
impl Scene {
  pub fn observe<C: Component, F: Fn(&Node, &C) + 'static>(&mut self, f: F) {
    let type_id = TypeId::of::<C>();
    let obs = Box::new(Observer::<C> { cb: Box::new(f) });
    match self.observers.get_mut(&type_id) {
      Some(observers) => observers.push(obs),
      None => {
        self.observers.insert(type_id, vec![obs]);
      }
    }
  }
  pub fn new() -> Self {
    let mut world = World::new();
    world.register::<Relationship>();
    world.register::<GeomSphere>();
    world.register::<Transform>();
    world.register::<Camera>();
    world.register::<Mesh>();
    world.register::<crate::Test>();

    let handle = Arc::new(RwLock::new(world));
    let root = Node {
      entity: handle
        .write()
        .unwrap()
        .create_entity()
        .with::<Relationship>(Relationship::default())
        .build(),
      world: Arc::downgrade(&handle),
    };
    Self {
      world: handle,
      root,
      observers: HashMap::new(),
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
        .write()
        .unwrap()
        .create_entity()
        .with::<Relationship>(Relationship {
          parent,
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
  pub fn each<T: Component, F: FnMut(&T)>(&self, mut f: F) {
    let world = self.world.read().unwrap();
    for s in world.write_storage::<T>().join() {
      f(s);
    }
  }
}
impl Drop for Scene {
  fn drop(&mut self) {
    self
      .world
      .write()
      .unwrap()
      .delete_entity(self.root.entity)
      .expect("Unable to delete entity");
  }
}

// trait IObserver {
//   fn update<C: Component>(&mut self, node: &Node, component: &C);
// }

// trait ISubject<'a, T: IObserver> {
//   fn attach(&mut self, observer: &'a mut T);
//   fn detach(&mut self, observer: &'a T);
//   fn notify_observers(&mut self);
// }

// struct Subject<'a, T: IObserver> {
//   observers: Vec<&'a mut T>,
// }
// impl<'a, T: IObserver + PartialEq> Subject<'a, T> {
//   fn new() -> Subject<'a, T> {
//     Subject {
//       observers: Vec::new(),
//     }
//   }
// }

// impl<'a, T: IObserver + PartialEq> ISubject<'a, T> for Subject<'a, T> {
//   fn attach(&mut self, observer: &'a mut T) {
//     self.observers.push(observer);
//   }
//   fn detach(&mut self, observer: &'a T) {
//     if let Some(idx) = self.observers.iter().position(|x| *x == observer) {
//       self.observers.remove(idx);
//     }
//   }
//   fn notify_observers(&mut self) {
//     for item in self.observers.iter_mut() {
//       let node = Node::new();
//       let test = Test {};
//       item.update(&node, &test);
//     }
//   }
// }

// #[derive(Component)]
// struct Test;

// #[derive(PartialEq)]
// struct ConcreteObserver {
//   id: i32,
// }
// impl IObserver for ConcreteObserver {
//   fn update<C: Component>(&mut self, node: &Node, component: &C) {
//     println!("Observer id:{} received event!", self.id);
//   }
// }

// #[test]
// fn test_observer() {
//   let mut subject = Subject::new();
//   let mut observer_a = ConcreteObserver { id: 1 };
//   let mut observer_b = ConcreteObserver { id: 2 };

//   subject.attach(&mut observer_a);
//   subject.attach(&mut observer_b);
//   subject.notify_observers();

//   subject.detach(&observer_b);
//   subject.notify_observers();
// }
