// use flux::ecs::{storage::Component, world::World};

// #[derive(Debug)]
// struct Position {
//   x: f32,
//   y: f32,
// }
// impl Component for Position {}

// #[derive(Debug)]
// struct Velocity {
//   x: f32,
//   y: f32,
// }
// impl Component for Velocity {}

// #[test]
// fn test_basic() {
//   let mut world = World::new();
//   for i in 0..10 {
//     let entity = world.create_entity();
//     world.add_component(
//       &entity,
//       Position {
//         x: (i * 10) as f32,
//         y: (i * 20) as f32,
//       },
//     );
//     world.add_component(
//       &entity,
//       Velocity {
//         x: (i * 1) as f32,
//         y: (i * 2) as f32,
//       },
//     );
//   }

//   world.each(|position: &Position| {
//     println!("{:?}", position);
//   });
//   // world.each(|position: &Position, velocity: &Velocity| {
//   //   println!("{:?}", (position, velocity));
//   // });
// }
