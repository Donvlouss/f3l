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

    let vertices = ply_wrapper.payload["vertex"]
        .iter()
        .map(|v| {
            let vertex = [v["x"].clone(), v["y"].clone(), v["z"].clone()];
            vertex
                .iter()
                .map(|v| match v {
                    Property::Float(f) => *f,
                    Property::Double(d) => *d as f32,
                    _ => 0f32,
                })
                .collect::<Vec<f32>>()
        })
        .collect::<Vec<Vec<f32>>>();

    vertices.into_iter().map(|v| [v[0], v[1], v[2]]).collect()
}

#[allow(dead_code)]
pub fn hsl_to_rgb(hsl: &[f32; 3]) -> [f32; 3] {
    let [h, s, l] = *hsl;
    let c = (1. - (2. * l - 1.).abs()) * s;
    let hh = h * 6.; // [0-1] * 360 / 60
    let x = c * (1. - (hh.rem_euclid(2.) - 1.).abs());
    let mut r = 0.;
    let mut g = 0.;
    let mut b = 0.;

    if hh >= 0. && h < 1. {
        r = c;
        g = x;
    } else if hh >= 1. && hh < 2. {
        r = x;
        g = c;
    } else if hh >= 2. && hh < 3. {
        g = c;
        b = x;
    } else if hh >= 3. && hh < 4. {
        g = x;
        b = c;
    } else if hh >= 4. && hh < 5. {
        r = x;
        b = c;
    } else {
        r = c;
        b = x;
    }
    [r, g, b]
}

#[allow(dead_code)]
pub fn hsv_to_rgb(hsv: &[f32; 3]) -> [f32; 3] {
    let [h, s, v] = *hsv;

    let hh = h * 6.;
    let hi = hh as usize;
    let f = hh - hi as f32;
    let p = v * (1. - s);
    let q = v * (1. - f * s);
    let t = v * (1. - (1. - f) * s);

    match hi {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => [0., 0., 0.],
    }
}

#[allow(dead_code)]
pub fn random_color() -> [f32; 3] {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    hsv_to_rgb(&[rng.gen_range(0f32..1f32), 1.0, 1.0])
}

#[allow(dead_code)]
fn main() {}
