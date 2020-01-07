use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::transform::Transform,
    core::timing::Time,
    ecs::prelude::{Component, DenseVecStorage, Entity},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

use crate::audio::initialize_sound;

pub const ARENA_HEIGHT: f32 = 100.0;
pub const ARENA_WIDTH: f32 = 100.0;

pub const PADDLE_HEIGHT: f32 = 16.0;
pub const PADDLE_WIDTH: f32 = 4.0;

pub const BALL_VELOCITY_X: f32 = 50.0;
pub const BALL_VELOCITY_Y: f32 = 50.0;
pub const BALL_SIZE: f32 = 2.0;

pub const MOVE_SCALE: f32 = 60.0;

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct Paddle {
    pub height: f32,
    pub width: f32,
    pub side: Side,
}

impl Paddle {
    fn new(side: Side) -> Self {
        Paddle {
            height: PADDLE_HEIGHT,
            width: PADDLE_WIDTH,
            side
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

pub struct Ball {
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub size: f32,
}

impl Ball {
    fn new() -> Self {
        Ball {
            velocity_x: BALL_VELOCITY_X,
            velocity_y: BALL_VELOCITY_Y,
            size: BALL_SIZE,
        }
    }
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: u32,
    pub score_right: u32,
}

pub struct ScoreText {
    pub p1_score: Entity,
    pub p2_score: Entity,
}

pub enum Pong {
    Uninitialized,
    Initialized {
        ball_spawn_timer: Option<f32>,
        sprite_sheet_handle: Handle<SpriteSheet>,
    }
}

impl Default for Pong {
    fn default() -> Self {
        Pong::Uninitialized
    }
}

impl SimpleState for Pong {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;
        let sprite_sheet_handle = load_sprite_sheet(world);
        *self = Pong::Initialized {
            ball_spawn_timer: Some(1.0),
            sprite_sheet_handle: sprite_sheet_handle.clone()
        };

        initialize_camera(world);
        initialize_paddles(world, sprite_sheet_handle);
        initialize_scoreboard(world);
        initialize_sound(world);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        match self {
            Pong::Uninitialized => (),
            Pong::Initialized { ball_spawn_timer, sprite_sheet_handle } => {
                if let Some(mut time) = ball_spawn_timer.take() {
                    time -= data.world.fetch::<Time>().delta_seconds();
                    if time <= 0.0 {
                        initialize_ball(data.world, sprite_sheet_handle.clone());
                    } else {
                        ball_spawn_timer.replace(time);
                    }
                }
            }
        };
        Trans::None
    }
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

fn initialize_paddles(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    let mut left_transform = Transform::default();
    let mut right_transform = Transform::default();

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0,
    };

    let y = ARENA_HEIGHT / 2.0;
    left_transform.set_translation_xyz(PADDLE_WIDTH * 0.5, y, 0.0);
    right_transform.set_translation_xyz(ARENA_WIDTH - PADDLE_WIDTH * 0.5, y, 0.0);

    world.create_entity()
        .with(Paddle::new(Side::Left))
        .with(left_transform)
        .with(sprite_render.clone())
        .build();

    world.create_entity()
        .with(Paddle::new(Side::Right))
        .with(right_transform)
        .with(sprite_render)
        .build();
}

fn initialize_ball(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    let mut transform = Transform::default();

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 1,
    };

    transform.set_translation_xyz(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0, 0.0);

    world.create_entity()
        .with(Ball::new())
        .with(transform)
        .with(sprite_render)
        .build();
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    let texture_handle = loader.load(
        "texture/pong_spritesheet.png",
        ImageFormat::default(),
        (),
        &texture_storage,
    );

    let loader = world.read_resource::<Loader>();
    let spritesheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
    let spritesheet_handle = loader.load(
        "texture/pong_spritesheet.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &spritesheet_storage,
    );
    spritesheet_handle
}

fn initialize_scoreboard(world: &mut World) {
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource()
    );

    let p1_transform = UiTransform::new(
        "P1".to_owned(), Anchor::TopMiddle, Anchor::TopMiddle,
        -50., -50., 1., 200., 50.,
    );

    let p2_transform = UiTransform::new(
        "P2".to_owned(), Anchor::TopMiddle, Anchor::TopMiddle,
        50., -50., 1., 200., 50.,
    );

    let p1_score = world
        .create_entity()
        .with(p1_transform)
        .with(UiText::new(font.clone(), "0".to_owned(), [1., 1., 1., 1.], 50.))
        .build();

    let p2_score = world
        .create_entity()
        .with(p2_transform)
        .with(UiText::new(font, "0".to_owned(), [1., 1., 1., 1.], 50.))
        .build();

    world.insert(ScoreText {p1_score, p2_score});

}
