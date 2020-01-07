use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    core::SystemDesc,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, World, WriteStorage},
};

use crate::pong::Ball;

#[derive(SystemDesc)]
pub struct MoveSystem;

impl <'s> System<'s> for MoveSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Ball>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut transforms, balls, time): Self::SystemData) {
        for (transform, ball) in (&mut transforms, &balls).join() {
            transform.prepend_translation_x(ball.velocity_x * time.delta_seconds());
            transform.prepend_translation_y(ball.velocity_y * time.delta_seconds());
        }
    }
}
