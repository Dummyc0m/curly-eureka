use std::ops::Deref;

use amethyst::{
    core::transform::Transform,
    core::SystemDesc,
    derive::SystemDesc,
    ecs::prelude::{Join, System, SystemData, World, WriteStorage, Write, ReadExpect},
    ui::{UiText},
};

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    ecs::{Read},
};

use crate::audio::{Sounds};
use crate::pong::{Ball, ARENA_WIDTH, ScoreBoard, ScoreText};

#[derive(SystemDesc)]
pub struct WinnerSystem;

impl <'s> System<'s> for WinnerSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, UiText>,
        Write<'s, ScoreBoard>,
        ReadExpect<'s, ScoreText>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>
    );
    fn run(&mut self, (mut balls, mut transforms, mut ui_texts, mut score_board, score_text, source_storage, sounds, output): Self::SystemData) {
        for (ball, transform) in (&mut balls, &mut transforms).join() {
            let hit = if transform.translation().x > ARENA_WIDTH {
                println!("p1 wins");
                score_board.score_left = (score_board.score_left + 1).min(999);
                if let Some(text) = ui_texts.get_mut(score_text.p1_score) {
                    text.text = score_board.score_left.to_string();
                }
                sounds.play_score_sound(&*source_storage, output.as_ref().map(|s| s.deref()));
                true
            } else if transform.translation().x < 0.0 {
                println!("p2 wins");
                score_board.score_right = (score_board.score_right + 1).min(999);
                if let Some(text) = ui_texts.get_mut(score_text.p2_score) {
                    text.text = score_board.score_right.to_string();
                }
                sounds.play_score_sound(&*source_storage, output.as_ref().map(|s| s.deref()));
                true
            } else {
                false
            };
            if hit {
                ball.velocity_x = -ball.velocity_x;
                transform.set_translation_x(ARENA_WIDTH / 2.0);
            }
        }
    }
}
