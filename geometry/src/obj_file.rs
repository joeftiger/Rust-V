use crate::Face;
use crate::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::SplitWhitespace;

#[derive(Serialize, Deserialize)]
pub struct ObjFile {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl ObjFile {
    pub fn new(vertices: Vec<Vertex>, faces: Vec<Face>) -> Self {
        Self { vertices, faces }
    }
}

impl From<ObjFile> for String {
    fn from(obj_file: ObjFile) -> Self {
        let v: Vec<String> = obj_file
            .vertices
            .iter()
            .map(|v| {
                let p = v.position;
                let n = v.normal;
                format!(
                    "v {0} {1} {2}\nvn {3} {4} {5}",
                    p.x, p.y, p.z, n.x, n.y, n.z
                )
            })
            .collect();

        // offset by one because indexing starts at 1 in obj files
        let mut f: Vec<String> = obj_file
            .faces
            .iter()
            .map(|f| format!("f {0} {1} {2}", f.v.0 + 1, f.v.1 + 1, f.v.2 + 1))
            .collect();

        let mut out = v;
        out.append(&mut f);
        out.join("\n")
    }
}

impl<P> From<P> for ObjFile
where
    P: AsRef<Path>,
{
    fn from(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Could not load path");

        let mut vertices = Vec::new();
        let mut faces = Vec::new();

        for (line_number, line_content) in content.lines().enumerate() {
            if line_content.starts_with('#') || line_content.is_empty() {
                continue;
            }

            let mut iter = line_content.split_whitespace();

            let id = iter
                .next()
                .unwrap_or_else(|| panic!("Invalid length at line {}", line_number));

            match id {
                "v" => {
                    let position = parse_vector3(&mut iter);
                    let vertex = Vertex {
                        position,
                        normal: Vector3::zero(),
                    };
                    vertices.push(vertex);
                }
                "f" => {
                    let face = parse_face(&mut iter);
                    faces.push(face);
                }
                _ => {} //eprintln!("Unsupported (skipping): {}", id),
            }
        }

        // initialize face normals
        faces.iter_mut().for_each(|f| {
            let (v0, v1, v2) = f.get_vertices(&vertices);

            f.normal = (v1.position - v0.position)
                .cross(v2.position - v0.position)
                .normalized()
        });

        // compute face normals and add them to vertices
        for f in &faces {
            let (v0, v1, v2) = f.get_vertices(&mut vertices);
            let (w0, w1, w2) = Mesh::angle_weights(v0.position, v1.position, v2.position);

            // scatter face normals to vertex normals
            vertices[f.v.0 as usize].normal += w0 * f.normal;
            vertices[f.v.1 as usize].normal += w1 * f.normal;
            vertices[f.v.2 as usize].normal += w2 * f.normal;
        }

        // normalize vertex normals
        vertices.iter_mut().for_each(|v| v.normal.normalize());

        Self::new(vertices, faces)
    }
}

fn parse_vector3(iter: &mut SplitWhitespace) -> Vector3 {
    let x = iter.next().unwrap().parse().unwrap();
    let y = iter.next().unwrap().parse().unwrap();
    let z = iter.next().unwrap().parse().unwrap();

    Vector3::new(x, y, z)
}

fn parse_face(iter: &mut SplitWhitespace) -> Face {
    let p = |s: &str| -> u32 { s.splitn(2, '/').next().unwrap().parse().unwrap() };

    let v0 = p(iter.next().unwrap());
    let v1 = p(iter.next().unwrap());
    let v2 = p(iter.next().unwrap());

    // offset by one because indexing starts at 1 in obj files
    let vertices = (v0 - 1, v1 - 1, v2 - 1);

    Face::new(vertices, Vector3::zero())
}
