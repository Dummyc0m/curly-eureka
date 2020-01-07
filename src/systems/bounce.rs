use std::ops::Deref;
use amethyst::{
    core::{Transform, SystemDesc},
    derive::SystemDesc,
    ecs::prelude::{Join, ReadStorage, System, SystemData, World, WriteStorage},
};

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    ecs::{Read, ReadExpect},
};

use crate::audio::{Sounds};

use crate::pong::{Paddle, Ball, Side, ARENA_HEIGHT};

#[derive(SystemDesc)]
pub struct BounceSystem;

impl <'s> System<'s> for BounceSystem {
    type SystemData = (
        ReadStorage<'s, Paddle>,
        WriteStorage<'s, Ball>,
        ReadStorage<'s, Transform>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>
    );
    fn run(&mut self, (paddles, mut balls, transforms, source_storage, sounds, output): Self::SystemData) {
        for (ball, transform) in (&mut balls, &transforms).join() {
            let ball_x = transform.translation().x;
            let ball_y = transform.translation().y;

            for (paddle, paddle_transform) in (&paddles, &transforms).join() {
                let paddle_x = paddle_transform.translation().x;
                let paddle_y = paddle_transform.translation().y;

                if point_in_rect(ball_x, ball_y, paddle_x, paddle_y, paddle.width / 2.0 + ball.size, paddle.height / 2.0 + ball.size)
                    && (ball.velocity_x > 0.0 && paddle.side == Side::Right
                        || ball.velocity_x < 0.0 && paddle.side == Side::Left
                    ) {
                            ball.velocity_x = -ball.velocity_x;
                            sounds.play_bounce_sound(&*source_storage, output.as_ref().map(|s| s.deref()));
                    }
            }

            if ball_y < ball.size && ball.velocity_y < 0.0
                || ball_y > ARENA_HEIGHT - ball.size && ball.velocity_y > 0.0 {
                    ball.velocity_y = -ball.velocity_y;
                    sounds.play_bounce_sound(&*source_storage, output.as_ref().map(|s| s.deref()));
                }
        }
    }
}

fn point_in_rect(p_x: f32, p_y: f32, paddle_x: f32, paddle_y: f32, h_tolerance: f32, v_tolerance: f32) -> bool {
    (p_x - paddle_x).abs() <= h_tolerance
        && (p_y - paddle_y).abs() <= v_tolerance
}
