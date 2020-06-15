use amethyst::{
  prelude::*,
  SimpleState, StateData, GameData, Trans, SimpleTrans, StateEvent, winit,
  assets::{
    Loader, AssetStorage, Handle
  },
  input::{ is_close_requested, is_key_down },
  renderer::{
    resources::AmbientColor,
    types::{Mesh, MeshData},
    light::{Light, DirectionalLight, PointLight},
    rendy::mesh::{ MeshBuilder, Position, Color, TexCoord, Indices},
    rendy::util::types::vertex::{PosColor, PosTex},
    debug_drawing::DebugLines,
    palette::{ LinSrgba, Srgb, Srgba },
    visibility::BoundingSphere,
    MaterialDefaults,
    ActiveCamera,
    loaders::load_from_linear_rgba,
    Camera,
    Material
  },
  controls::FlyControlTag,
  utils::auto_fov::AutoFov,
  core::{
    transform::Transform,
    math::base::Vector3
  },
};

use crate::terrain::terrain_gen::{TerrainGenerator, WIDTH, DEPTH};
use crate::terrain::surface_net::SurfaceNet;
use crate::config::generator::GeneratorConfig;
use std::fs::File;
use std::io::Write;
use ron::ser::PrettyConfig;
use crate::util::calculate_normals;

#[derive(Default)]
pub struct March;

impl SimpleState for March {
  fn on_start(&mut self, data: StateData<GameData>) {
    initialize_triangle(data.world);
    initialize_terrain(data.world);
    initialize_ambient_light(data.world);
    initialize_directional_light(data.world);
    initialize_camera(data.world);
  }

  fn handle_event(
    &mut self,
    data: StateData<'_, GameData<'_, '_>>,
    event: StateEvent,
  ) -> SimpleTrans {
    let StateData { world, .. } = data;
    if let StateEvent::Window(ref event) = event {
      if is_close_requested(&event) || is_key_down(&event, winit::VirtualKeyCode::Escape) {
        Trans::Quit
      } else {
        Trans::None
      }
    } else {
      Trans::None
    }
  }

  fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
    Trans::None
  }

}

fn initialize_camera(world: &mut World) {
  let mut transform = Transform::default();
  transform.set_translation_xyz(0.0, 80.0, 0.0);

  let mut auto_fov = AutoFov::default();
  auto_fov.set_base_fovx(std::f32::consts::FRAC_PI_3);
  auto_fov.set_base_aspect_ratio(1, 1);

  let camera = world
    .create_entity()
    .with(Camera::standard_3d(16.0, 9.0))
    .with(auto_fov)
    .with(transform)
    .with(FlyControlTag)
    .build();

  world.insert(ActiveCamera {
    entity: Some(camera),
  });
  world.insert(DebugLines::new());
}

fn initialize_directional_light(world: &mut World) {
  let light = DirectionalLight {
    intensity: 0.7,
    direction: Vector3::new(-0.3, -1.0, -0.3),
    color: Srgb::new(1.0, 1.0, 1.0),
  };
  let light: Light = light.into();

  let mut light_transform = Transform::default();
  light_transform.set_translation_y(10.0);
  world.create_entity()
    .with(light)
    .with(light_transform)
    .build()
  ;
}

fn initialize_ambient_light(world: &mut World) {
  world.exec(|mut color: amethyst::ecs::prelude::Write<'_, AmbientColor>| {
    color.0 = Srgba::new(0.2, 0.2, 0.2, 1.0);
  })
}

fn initialize_triangle(world: &mut World) {
  let vertices = vec![
    // Bottom left
      Position([-1.0, -1.0, 0.0]),
    // Bottom right
      Position([1.0, -1.0, 0.0]),
    // Top middle
      Position([0.0, 1.0, 0.0]),
  ];
  let tex_coords = vec![
    // Bottom left
      TexCoord([1.0, 0.0]),
    // Bottom right
      TexCoord([1.0, 0.0]),
    // Top middle
      TexCoord([1.0, 0.0]),
  ];
  let triangles = vec![
    0_u16, 1, 2
  ];
  let normals = calculate_normals(&vertices, &triangles);
  let mesh_data = MeshData(
    MeshBuilder::new()
      .with_vertices(vertices)
      .with_vertices(normals)
      .with_vertices(tex_coords)
      .with_indices(triangles)
  );
  let mesh_handle = world.read_resource::<Loader>().load_from_data(mesh_data, (), &world.read_resource::<AssetStorage<Mesh>>());
  let mat_handle = mk_material(world);
  world.create_entity()
    .with(mesh_handle)
    .with(mat_handle)
    .with(Transform::from(Vector3::new(-1.0, -1.0, -1.0)))
    .build();
}

fn mk_material(world: &mut World) -> Handle<Material> {
  let color = load_from_linear_rgba(LinSrgba::new(1.0, 0.0, 0.5, 1.0));
  let texture_handle = world.read_resource::<Loader>().load_from_data(color.into(), (), &world.read_resource());

  let default_mat = world.read_resource::<MaterialDefaults>().0.clone();

  let mat = Material {
    albedo: texture_handle,
    ..default_mat
  };
  let mat_handle = world.read_resource::<Loader>().load_from_data(mat, (), &world.read_resource::<AssetStorage<Material>>());
  mat_handle
}

fn initialize_terrain(world: &mut World) {
  // let default_mat = world.read_resource::<MaterialDefaults>().0.clone();
  let gen_config = (&*world.read_resource::<GeneratorConfig>()).clone();
  println!("generating terrain");
  let terrain_gen = TerrainGenerator::new(gen_config);
  let mat_handle = mk_material(world);
  for i in -5..5 {
    for j in -5..5 {
      initialize_chunk(world, &terrain_gen, mat_handle.clone(), (i, j))
    }
  }
}

fn initialize_chunk(world: &mut World, terrain_gen: &TerrainGenerator, mat_handle: Handle<Material>, (x, z): (i32, i32)) {
  println!("generating chunk ({}, {})", x, z);
  let chunk = terrain_gen.generate_chunk(x, z);
  println!("generating surface net");
  let surface_net = SurfaceNet::new();
  println!("generating surface net cubes");
  let cubes = surface_net.mk_cubes(&chunk);
  println!("generating mesh");
  let mesh_data = surface_net.mk_mesh(cubes);

  println!("loading mesh");
  let mesh_handle = {
    let loader = world.read_resource::<Loader>();
    let mesh_asset_storage = world.read_resource::<AssetStorage<Mesh>>();
    let mesh_handle = loader.load_from_data(mesh_data, (), &*mesh_asset_storage);
    mesh_handle
  };

  let mut transform = Transform::from(Vector3::new((x * WIDTH as i32) as f32, 0.0, (z * DEPTH as i32) as f32));

  println!("creating entity");
  world.create_entity()
    .with(mesh_handle)
    .with(BoundingSphere::origin(256.0))
    .with(mat_handle)
    .with(transform)
    .build();
}
