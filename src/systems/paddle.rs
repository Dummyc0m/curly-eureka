use crate::pong::PADDLE_HEIGHT;
use crate::pong::ARENA_HEIGHT;
use amethyst::core::{Transform, SystemDesc, timing::Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::{Join, Read, ReadStorage, System, SystemData, World, WriteStorage};
use amethyst::input::{InputHandler, StringBindings};

use crate::pong::{Paddle, Side, MOVE_SCALE};

#[derive(SystemDesc)]
pub struct PaddleSystem;

impl<'s> System<'s> for PaddleSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Paddle>,
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, Time>
    );

    fn run(&mut self, (mut transforms, paddles, input, time): Self::SystemData) {
        for (paddle, transform) in (&paddles, &mut transforms).join() {
            let (movement, name) = match paddle.side {
                Side::Left => (input.axis_value("left_paddle"), "left"),
                Side::Right => (input.axis_value("right_paddle"), "right"),
            };

            if let Some(move_amount) = movement {
                if move_amount != 0.0 {
                    let scaled_amount = move_amount as f32 * MOVE_SCALE * time.delta_seconds();
                    let paddle_y = transform.translation().y;
                    transform.set_translation_y((scaled_amount + paddle_y)
                                                .min(ARENA_HEIGHT - PADDLE_HEIGHT * 0.5)
                                                .max(PADDLE_HEIGHT * 0.5));
                }
            }
        }
    }
}
