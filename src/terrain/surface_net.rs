use amethyst::core::math::base::{Matrix3, Vector3, Vector4};
use amethyst::renderer::{
  types::{Mesh, MeshData},
  shape::Shape,
  rendy::{
    mesh::{ MeshBuilder, Position, Color, Indices, TexCoord },
    util::types::vertex::PosColor,
  },
};
use getset::{Getters};
use ndarray::Array3;
use num_traits::identities::Zero;
use crate::terrain::terrain_gen::{Chunk, WIDTH, DEPTH, HEIGHT};
use std::fs::File;
use std::io::Write;
use ron::ser::PrettyConfig;
use crate::util::calculate_normals;

/// ripping off of https://github.com/mikolalysenko/mikolalysenko.github.com/blob/master/Isosurface/js/surfacenets.js
/// https://0fps.net/2012/07/12/smooth-voxel-terrain-part-2/
#[derive(Getters, Clone, Debug, PartialEq)]
pub struct SurfaceNetCube {
  position: Vector3<f32>,
  on_surface: bool,
  corner_mask: u8,
}

impl Default for SurfaceNetCube {
  fn default() -> Self {
    SurfaceNetCube {
      position: Vector3::zero(),
      on_surface: false,
      corner_mask: 0,
    }
  }
}

pub struct SurfaceNet {
  voxel_corner_offsets: [Vector3<usize>; 8],
  voxel_corner_offsets_f32: [Vector3<f32>; 8],
  cube_edges: [u32; 24],
  intersection_table: [u32; 256],
}

impl SurfaceNet {
  fn mk_cube_edges() -> [u32; 24] {
    let mut cube_edges = [0_u32; 24];
    let mut k = 0;
    for i in 0_u32..8 {
      let mut j = 1_u32;
      while j <= 4 {
        let p = i ^ j;
        if i <= p {
          cube_edges[k] = i;
          k += 1;
          cube_edges[k] = p;
          k += 1;
        }

        j <<= 1;
      }
    }

    return cube_edges;
  }

  fn mk_intersection_table(cube_edges: &[u32; 24]) -> [u32; 256] {
    let mut edge_table = [0_u32; 256];
    for i in 0..256 {
      let mut em = 0;
      let mut j = 0;
      while j < 24 {
        let a = (i & (1 << cube_edges[j])) != 0;
        let b = (i & (1 << cube_edges[j + 1])) != 0;
        em |= if a != b { (1 << (j >> 1)) } else { 0 };

        j += 2;
      }
      edge_table[i] = em as u32;
    }
    edge_table
  }

  pub fn new() -> Self {
    /*
    *  y         z
    *  ^        /
    *  |
    *    6----7
    *   /|   /|
    *  4----5 |
    *  | 2--|-3
    *  |/   |/
    *  0----1   --> x
    *
    */
    let voxel_corner_offsets: [Vector3<usize>; 8] = [
      Vector3::new (0, 0, 0), // 0
      Vector3::new (1, 0, 0), // 1
      Vector3::new (0, 1, 0), // 2
      Vector3::new (1, 1, 0), // 3
      Vector3::new (0, 0, 1), // 4
      Vector3::new (1, 0, 1), // 5
      Vector3::new (0, 1, 1), // 6
      Vector3::new (1, 1, 1), // 7
    ];

    let voxel_corner_offsets_f32: [Vector3<f32>; 8] = [
      Vector3::new (0.0, 0.0, 0.0), // 0
      Vector3::new (1.0, 0.0, 0.0), // 1
      Vector3::new (0.0, 1.0, 0.0), // 2
      Vector3::new (1.0, 1.0, 0.0), // 3
      Vector3::new (0.0, 0.0, 1.0), // 4
      Vector3::new (1.0, 0.0, 1.0), // 5
      Vector3::new (0.0, 1.0, 1.0), // 6
      Vector3::new (1.0, 1.0, 1.0), // 7
    ];

    let cube_edges = Self::mk_cube_edges();
    let intersection_table = Self::mk_intersection_table(&cube_edges);


    SurfaceNet {
      voxel_corner_offsets,
      voxel_corner_offsets_f32,
      cube_edges,
      intersection_table
    }
  }

  pub fn mk_cubes(&self, chunk: &Chunk) -> Array3<SurfaceNetCube> {
    let mut cubes = Array3::from_elem((WIDTH, HEIGHT, DEPTH), SurfaceNetCube::default());
    for x in 0..(WIDTH - 1) {
      for y in 0..(HEIGHT - 1) {
        for z in 0..(DEPTH - 1) {
          let mut sample = [0f32; 8];
          for i in 0..8 {
            let offset = self.voxel_corner_offsets[i];
            let value = chunk.get((x + offset.index(0), y + offset.index(1), z + offset.index(2))).unwrap();
            sample[i] = value;
          }
          let SurfaceNetCube { position, corner_mask, on_surface } = self.mk_surface_net_cube(sample);
          cubes[(x, y, z)] = SurfaceNetCube {
            position: position + Vector3::new(x as f32, y as f32, z as f32),
            corner_mask,
            on_surface,
          }
        }
      }
    }
    cubes
  }

  pub fn mk_surface_net_cube(&self, sample: [f32; 8]) -> SurfaceNetCube {
    // create corner mask
    let mut corner_mask = 0_u8;
    for i in 0..8 {
      corner_mask |= if sample[i] > 0.0 { (1 << i) as u8 } else { 0 };
    }

    // interior cube
    if corner_mask == 0 || corner_mask == 0xff {
      return SurfaceNetCube::default();
    }

    let edge_mask = self.intersection_table[corner_mask as usize];
    let mut edge_crossings = 0;
    let mut vert_pos = Vector3::zero();

    for i in 0..12 {
      //Use edge mask to check if it is crossed
      if !((edge_mask & (1 << i)) > 0) {
        continue;
      }
      //If it did, increment number of edge crossings
      edge_crossings += 1;

      //Now find the point of intersection
      let e0 = self.cube_edges[i << 1];
      let e1 = self.cube_edges[(i << 1) + 1];
      let g0 = sample[e0 as usize];
      let g1 = sample[e1 as usize];
      let t = (-g0) / (g1 - g0);

      vert_pos += (self.voxel_corner_offsets_f32[e0 as usize]).lerp(&(self.voxel_corner_offsets_f32[e1 as usize]), t);
    }

    vert_pos /= edge_crossings as f32;

    SurfaceNetCube  {
      position: vert_pos,
      corner_mask,
      on_surface: true,
    }
  }

  pub fn mk_mesh(&self, cubes: Array3<SurfaceNetCube>) -> MeshData {
    let mut vertices = Vec::<Vector3<f32>>::new();
    let mut triangles = Vec::<u16>::new();
    let mut tex_coords = Vec::<TexCoord>::new();

    let (width, height, depth) = cubes.dim();
    let mut pos = [0; 3];
    let mut r = [1, width as i32 + 1, (width as i32 + 1) * (height as i32 + 1)];
    let mut buf_no = 1;

    let mut vertex_buffer = vec![0_usize; r[2] as usize * 2];

    while pos[2] < depth - 1 {
      let mut buf_idx = 1 + (width + 1) * ( 1 + buf_no * (height + 1));

      pos[1] = 0;
      while pos[1] < height - 1 {
        pos[0] = 0;
        while pos[0] < width - 1 {
          let SurfaceNetCube { position, corner_mask, on_surface} =
            *cubes.get((pos[0], pos[1], pos[2])).unwrap();
          if !on_surface {
            pos[0] += 1;
            buf_idx += 1;
            continue
          }

          vertex_buffer[buf_idx] = vertices.len();
          vertices.push(position);
          // TODO add colors
          tex_coords.push(TexCoord([1.0, 0.0]));

          let edge_mask = self.intersection_table[corner_mask as usize];
          // add faces
          for i in 0..3 {
            // first 3 entries indicate crossing on edge
            if edge_mask & (1 << i) == 0 {
              continue
            }

            // i - Axes, iu, iv - Ortho Axes
            let iu = (i + 1) % 3;
            let iv = (i + 2) % 3;

            if pos[iu] == 0 || pos[iv] == 0 {
              continue
            }

            let du = r[iu];
            let dv = r[iv];

            //Flip Orientation Depending on Corner Sign
            if (corner_mask & 1) != 0 {
              Self::add_quad(vertex_buffer[buf_idx], vertex_buffer[(buf_idx as i32 - du) as usize],
                             vertex_buffer[(buf_idx as i32 - dv - du) as usize],
                             vertex_buffer[(buf_idx as i32 - dv) as usize], &vertices, &mut triangles,
              );
            } else {
              Self::add_quad(vertex_buffer[buf_idx], vertex_buffer[(buf_idx as i32 - dv) as usize],
                             vertex_buffer[(buf_idx as i32 - dv - du) as usize],
                             vertex_buffer[(buf_idx as i32 - du) as usize], &vertices, &mut triangles,
              );
            }
          }

          // increment
          pos[0] += 1;
          buf_idx += 1;
        }
        // increment
        pos[1] += 1;
        buf_idx += 2;
      }

      // increment
      pos[2] += 1;
      buf_no ^= 1;
      r[2] = -r[2];
    }

    // calculate normals

//    println!("serializing mesh");
//    let mut file = File::create("surface_net.vertices.ron").unwrap();
//    write!(file, "{}", ron::ser::to_string_pretty(&vertices, PrettyConfig::default()).unwrap()).unwrap();
//
//    let mut file = File::create("surface_net.tex_coords.ron").unwrap();
//    write!(file, "{}", ron::ser::to_string_pretty(&tex_coords, PrettyConfig::default()).unwrap()).unwrap();
//
//    let mut file = File::create("surface_net.triangles.ron").unwrap();
//    write!(file, "{}", ron::ser::to_string_pretty(&triangles, PrettyConfig::default()).unwrap()).unwrap();

    let positions = vertices.into_iter().map(|v| Position(v.into())).collect::<Vec<_>>();
    let normals = calculate_normals(&positions, &triangles);

    MeshData(
      MeshBuilder::new()
        .with_vertices(positions)
        .with_vertices(normals)
        .with_vertices(tex_coords)
        .with_indices(triangles)
    )
  }

  fn add_quad(a: usize, b: usize, c: usize, d: usize,
              vertices: &[Vector3<f32>], triangles: &mut Vec<u16>) {
    let vec_a = vertices[a];
    let vec_b = vertices[b];
    let vec_c = vertices[c];
    let vec_d = vertices[d];
    if (vec_a - vec_c).norm_squared() >= (vec_b - vec_d).norm_squared() {
      triangles.push(a as u16);
      triangles.push(b as u16);
      triangles.push(d as u16);

      triangles.push(d as u16);
      triangles.push(b as u16);
      triangles.push(c as u16);
    } else {
      triangles.push(a as u16);
      triangles.push(b as u16);
      triangles.push(c as u16);

      triangles.push(a as u16);
      triangles.push(c as u16);
      triangles.push(d as u16);
    }
  }
}