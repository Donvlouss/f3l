use ply_rs as ply;
use ply_rs::ply::Property;

pub fn load_ply(path: &str) -> Vec<[f32; 3]> {
    let mut f = std::fs::File::open(path).unwrap();
    // create a parser
    let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    // use the parser: read the entire file
    let ply = p.read_ply(&mut f);
    // make sure it did work
    assert!(ply.is_ok());

    let ply_wrapper = ply.unwrap();

    let vertices = ply_wrapper.payload["vertex"].iter()
        .map(|v| {
            let vertex = [v["x"].clone(), v["y"].clone(), v["z"].clone()];
            vertex.iter()
                .map(|v| {
                    match v {
                        Property::Float(f) => *f,
                        Property::Double(d) => *d as f32,
                        _ => 0f32
                    }
                }).collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    vertices.into_iter()
        .map(|v| {
            [v[0], v[1], v[2]]
        })
        .collect()
}

fn main() {}