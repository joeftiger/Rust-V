use crate::debug_util::is_normalized;
use crate::Face;
use definitions::Vector3;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::SplitWhitespace;

#[derive(Serialize, Deserialize)]
pub struct ObjFile {
    pub vertices: Vec<Vector3>,
    pub vertex_normals: Vec<Vector3>,
    pub faces: Vec<Face>,
}

impl ObjFile {
    pub fn new(vertices: Vec<Vector3>, vertex_normals: Vec<Vector3>, faces: Vec<Face>) -> Self {
        Self {
            vertices,
            vertex_normals,
            faces,
        }
    }

    pub fn has_vertex_normals(&self) -> bool {
        !self.vertex_normals.is_empty()
    }
}

impl From<ObjFile> for String {
    fn from(obj_file: ObjFile) -> Self {
        let v: Vec<String> = obj_file
            .vertices
            .iter()
            .map(|v| format!("v {} {} {}", v.x, v.y, v.z))
            .collect();

        // offset by one because indexing starts at 1 in obj files
        let mut f: Vec<String> = obj_file
            .faces
            .iter()
            .map(|f| {
                if let Some(n) = f.vn {
                    format!(
                        "f {}//{} {}//{} {}//{}",
                        f.v.0 + 1,
                        n.0 + 1,
                        f.v.1 + 1,
                        n.1 + 1,
                        f.v.2 + 1,
                        n.2 + 1
                    )
                } else {
                    format!("f {} {} {}", f.v.0 + 1, f.v.1 + 1, f.v.2 + 1)
                }
            })
            .collect();

        let mut vn = obj_file
            .vertex_normals
            .iter()
            .map(|vn| format!("vn {} {} {}", vn.x, vn.y, vn.z))
            .collect();

        let mut out = v;
        out.append(&mut vn);
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
        let mut vertex_normals = Vec::new();
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
                    let vertex = parse_vector3(&mut iter);
                    vertices.push(vertex);
                }
                "vn" => {
                    let mut normal = parse_vector3(&mut iter);
                    if !is_normalized(&normal) {
                        eprintln!(
                            "Vertex normal at line {} is not normalized: {:?}",
                            line_number, normal
                        );
                        normal.normalize();
                    }
                    vertex_normals.push(normal);
                }
                "f" => {
                    let face = parse_face(&mut iter);
                    faces.push(face);
                }
                _ => eprintln!("Unsupported (skipping): {}", id),
            }
        }

        Self::new(vertices, vertex_normals, faces)
    }
}

fn parse_vector3(iter: &mut SplitWhitespace) -> Vector3 {
    let x = iter.next().unwrap().parse().unwrap();
    let y = iter.next().unwrap().parse().unwrap();
    let z = iter.next().unwrap().parse().unwrap();

    Vector3::new(x, y, z)
}

fn parse_face(iter: &mut SplitWhitespace) -> Face {
    let p = |s: &str| -> (u32, Option<u32>) {
        if s.contains("//") {
            let i = s.splitn(2, "//").collect::<Vec<_>>();
            (
                i[0].parse()
                    .expect(&*format!("Unable to parse {:?} as a face", i)),
                Some(
                    i[1].parse()
                        .expect(&*format!("Unable to parse {:?} as a face", i)),
                ),
            )
        } else {
            (s.splitn(2, '/').next().unwrap().parse().unwrap(), None)
        }
    };

    let (v0, n0) = p(iter.next().unwrap());
    let (v1, n1) = p(iter.next().unwrap());
    let (v2, n2) = p(iter.next().unwrap());

    // offset by one because indexing starts at 1 in obj files
    let vertices = (v0 - 1, v1 - 1, v2 - 1);
    let normals = n0
        .zip(n1)
        .zip(n2)
        .map(|n| (n.0 .0 - 1, n.0 .1 - 1, n.1 - 1));

    Face::new(vertices, normals)
}
