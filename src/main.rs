use std::path::Path;
use amethyst::{
  core::{transform::TransformBundle, frame_limiter::FrameRateLimitStrategy},
  prelude::*,
  renderer::{
    plugins::{RenderShaded3D, RenderPbr3D, RenderFlat3D, RenderToWindow, RenderDebugLines, RenderSkybox},
    types::DefaultBackend,
    RenderingBundle,
    palette::Srgb,
  },
  utils::application_root_dir,
  input::{InputBundle, StringBindings},
  audio::AudioBundle,
  utils::{
    auto_fov::AutoFovSystem,
    fps_counter::FpsCounterBundle,
  },
  controls::FlyControlBundle,
  config::Config,
};

use amethyst::ui::{RenderUi, UiBundle};
use crate::march::March;
use crate::config::generator::GeneratorConfig;

mod march;
mod terrain;
mod util;
mod config;

fn main() -> amethyst::Result<()> {
  amethyst::start_logger(Default::default());

  let app_root = application_root_dir()?;
  let display_config_path = app_root.join("config").join("display.ron");

  let bindings_path = app_root.join("config").join("bindings.ron");
  let input_bundle = InputBundle::<StringBindings>::new()
    .with_bindings_from_file(bindings_path)?;

  let generator_path = app_root.join("config").join("generator.ron");
  let generator_config = GeneratorConfig::load(&generator_path)?;

  let game_data = GameDataBuilder::default()
    .with(AutoFovSystem::default(), "auto_fov", &[])
    .with_bundle(FpsCounterBundle::default())?
    .with_bundle(input_bundle)?
    .with_bundle(
      FlyControlBundle::<StringBindings>::new(
        Some("horizontal".into()),
        None,
        Some("vertical".into()),
      )
        .with_sensitivity(0.1, 0.1)
        .with_speed(5.),
    )?
    .with_bundle(TransformBundle::new().with_dep(&[
      "fly_movement",
      "free_rotation"
    ]))?
    .with_bundle(UiBundle::<StringBindings>::new())?
    .with_bundle(
      RenderingBundle::<DefaultBackend>::new()
        .with_plugin(RenderToWindow::from_config_path(display_config_path)?)
        .with_plugin(RenderShaded3D::default())
        .with_plugin(RenderDebugLines::default())
        .with_plugin(RenderSkybox::with_colors(
          Srgb::new(0.82, 0.51, 0.50),
          Srgb::new(0.18, 0.11, 0.85),
        )),
    )?
    ;
  let assets_dir = app_root.join("assets");
  let mut game : Application<_> = ApplicationBuilder::new(assets_dir, March::default())
    ?.with_resource(generator_config)
    .with_frame_limit(FrameRateLimitStrategy::Yield, 60)
    .build(game_data)?;
  game.run();
  Ok(())
}
