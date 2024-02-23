#[cfg(feature = "app_bevy")]
use bevy::{
    prelude::*,
    render::view::RenderLayers
};
#[cfg(feature = "app_bevy")]
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin
};
#[cfg(feature = "app_bevy")]
use bevy_mod_picking::prelude::*;
#[cfg(feature = "app_bevy")]
use bevy_viewcube::prelude::*;
#[cfg(feature = "app_bevy")]
use bevy_points::{
    prelude::*,
    material::PointsShaderSettings
};
#[cfg(feature = "app_bevy")]
use bevy_egui::{
    egui,
    EguiContexts,
    EguiPlugin
};

#[cfg(feature = "app_kiss3d")]
use kiss3d::light::Light;
#[cfg(feature = "app_kiss3d")]
use kiss3d::window::Window;
#[cfg(feature = "app_kiss3d")]
use nalgebra::Point3;

use ply_rs as ply;
use ply_rs::ply::Property;
use f3l_filter::*;

#[cfg(not(feature = "app_bevy"))]
#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Add --features=<app_kiss3d or app_bevy>")
}

#[cfg(feature = "app_bevy")]
#[cfg(feature = "app_kiss3d")]
fn main() {
    println!(r"Add --features=<app_kiss3d or app_bevy>")
}

#[cfg(not(feature = "app_bevy"))]
#[cfg(feature = "app_kiss3d")]
fn main() {
    println!("Using Kiss3d app");

    let mut window = Window::new("Kiss3d: points");

    window.set_light(Light::StickToCamera);
    window.set_point_size(10.0); // (Not supported by all graphic drivers)

    let vertices = load_ply("E:/workspace/Rust/f3l/crates/f3l_filter/data/Itable_scene_lms400.ply");
    let mut filter = RadiusOutlierRemoval::with_data(5f32, 5, &vertices);
    let out = filter.filter_instance();

    while window.render() {
        let a = Point3::new(-0.1, -0.1, 0.0);
        let b = Point3::new(0.0, 0.1, 0.0);
        let c = Point3::new(0.1, -0.1, 0.0);

        window.draw_point(&a, &Point3::new(1.0, 0.0, 0.0));
        window.draw_point(&b, &Point3::new(0.0, 1.0, 0.0));
        window.draw_point(&c, &Point3::new(0.0, 0.0, 1.0));

        vertices.iter()
            .for_each(|v| {
                window.draw_point(&Point3::new(v[0], v[1], v[2]), &Point3::new(1.0, 1.0, 1.0));
            });
        out.iter()
            .for_each(|v| {
                window.draw_point(&Point3::new(v[0], v[1], v[2]), &Point3::new(0.0, 1.0, 0.0));
            });
    }
}

#[cfg(feature = "app_bevy")]
#[cfg(not(feature = "app_kiss3d"))]
fn main() {
    println!("Using bevy app");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(PointsPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(BevyViewCubePlugin::default())
        .add_systems(Startup, setup)
        .run();
    println!("End");
}

fn load_ply(path: &str) -> Vec<[f32; 3]> {
    let mut f = std::fs::File::open(path).unwrap();
    // create a parser
    let p = ply::parser::Parser::<ply::ply::DefaultElement>::new();
    // use the parser: read the entire file
    let ply = p.read_ply(&mut f);
    // make sure it did work
    assert!(ply.is_ok());

    let ply_wrapper = ply.unwrap();
    let header_vertex = ply_wrapper.header.elements["vertex"].clone();
    let vx_type = header_vertex.properties["x"].clone().data_type;

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

#[cfg(feature = "app_bevy")]
#[cfg(not(feature = "app_kiss3d"))]
fn setup(
    mut commands: Commands,
    mut point_mesh: ResMut<Assets<Mesh>>,
    mut point_material: ResMut<Assets<PointsMaterial>>,
    mut contexts: EguiContexts,
) {
    println!("Setup");
    let vertices = load_ply("data/Itable_scene_lms400.ply");
    println!("NB: {}", vertices.len());

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                ..default()
            },
            ..default()
        },
        PanOrbitCamera {
            allow_upside_down: true,
            ..Default::default()
        },
        // Need set camera layer, or viewcube would be selected on this camera.
        RenderLayers::layer(0),
        // ViewcubeBinding,
    ));
    commands.spawn(DirectionalLightBundle {
        ..Default::default()
    });

    let bevy_points = vertices.iter()
        .map(|v| {
            Vec3::from_array(*v)
        }).collect::<Vec<Vec3>>();

    let points_mesh = PointsMesh { vertices: bevy_points, colors: None};


    let ori_id = commands.spawn(
        MaterialMeshBundle {
            mesh: point_mesh.add(points_mesh.into()),
            material: point_material.add(PointsMaterial {
                settings: PointsShaderSettings {
                    point_size: 0.1,
                    opacity: 1.0,
                    ..Default::default()
                },
                perspective: true,
                alpha_mode: AlphaMode::Blend,
                circle: true,
                ..Default::default()
            }),
            ..Default::default()
        }
    ).id();

    egui::Window::new("Filter")
        .show(contexts.ctx_mut(), |ui| {
            if (ui.button("RadiusRemove").clicked()) {
                let mut filter = RadiusOutlierRemoval::with_data(5f32, 5, &vertices);
                let out = filter.filter_instance();
                if out.is_empty() {
                    return;
                }
                commands.entity(ori_id).despawn();
                let out_vertices = out.into_iter()
                    .map(|v| Vec3::new(v[0], v[1], v[2])).collect::<Vec<Vec3>>();
                let mesh = PointsMesh { vertices: out_vertices, colors: None };
                commands.spawn(
                    MaterialMeshBundle {
                        mesh: point_mesh.add(mesh.into()),
                        material: point_material.add(PointsMaterial {
                            settings: PointsShaderSettings {
                                point_size: 0.1,
                                opacity: 1.0,
                                ..Default::default()
                            },
                            perspective: true,
                            alpha_mode: AlphaMode::Blend,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }
                );
            }
        });
}