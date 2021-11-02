use bevy::{
    core::Time,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::prelude::*,
    math::{Vec2, Vec3},
    prelude::{App, Assets, AssetServer, Transform},
    pbr2::{PbrBundle, AmbientLight, /*PointLightBundle, PointLight,*/ StandardMaterial, DirectionalLightBundle, DirectionalLight, DirectionalLightShadowMap},
    math::{Quat},
    PipelinedDefaultPlugins,
    input::keyboard::KeyCode,
    input::Input,
    render2::{
        camera::OrthographicProjection,
        color::Color,
        mesh::{shape, Mesh},
        //view::Msaa
    },
    window::{WindowDescriptor, WindowMode},
};
use wasm_bindgen::prelude::*;

// todo: use asset-server or something more sophisticated eventually.
// for now, just hack it up and toss the office-demo YAML into a big string
mod demo_world;
use demo_world::demo_office;

use bevy_egui::{egui, EguiContext, EguiPlugin};

use std::{
    env,
    fs::{File, metadata},
    path::Path,
};
use serde_yaml;

mod supercamera;
use supercamera::{SuperCameraPlugin, FlexibleProjection, ProjectionMode};


struct Vertex {
    x: f64,
    y: f64,
    name: String,
}

struct Lane {
    start: usize,
    end: usize,
}

struct Wall {
    start: usize,
    end: usize,
}

struct SiteMap {
    filename: String,
    site_name: String,
    vertices: Vec<Vertex>,
    lanes: Vec<Lane>,
    walls: Vec<Wall>,
}

impl Default for SiteMap {
    fn default() -> Self {
        SiteMap {
            filename: String::new(),
            site_name: String::new(),
            vertices: Vec::new(),
            lanes: Vec::new(),
            walls: Vec::new(),
        }
    }
}

impl SiteMap {
    fn load(&mut self, filename: String) {
        println!("SiteMap loading file: [{}]", filename); //{} = {:?}", args.len(), args);
        self.filename = filename;
        if !metadata(&self.filename).is_ok() {
            println!("could not open [{}]", &self.filename);
            return;
        }
        let file = File::open(&self.filename).expect("Could not open file");
        let doc: serde_yaml::Value = serde_yaml::from_reader(file).ok().unwrap();
        self.load_yaml(doc);
    }

    fn load_demo(
        &mut self,
    ) {
        // todo: use asset-server or something more sophisticated eventually.
        // for now, just hack it up and toss the office-demo YAML into a big string
        let result: serde_yaml::Result<serde_yaml::Value> = serde_yaml::from_str(&demo_office());
        if result.is_err() {
            println!("serde threw an error: {:?}", result.err());
        }
        else {
            let doc: serde_yaml::Value = serde_yaml::from_str(&demo_office()).ok().unwrap();
            self.load_yaml(doc);
        }
    }

    fn load_yaml(&mut self, doc: serde_yaml::Value) {
        self.site_name = doc["name"].as_str().unwrap().to_string();
        for (k, level_yaml) in doc["levels"].as_mapping().unwrap().iter() { //.iter() {
            println!("level name: [{}]", k.as_str().unwrap());
            for vertex_yaml in level_yaml["vertices"].as_sequence().unwrap() {
                let data = vertex_yaml.as_sequence().unwrap();
                let x = data[0].as_f64().unwrap();
                let y = data[1].as_f64().unwrap();
                let name = if data.len() > 3 { data[3].as_str().unwrap().to_string() } else { String::new() };
                let v = Vertex {
                    x: x,
                    y: -y,
                    name: name
                };
                self.vertices.push(v);
            }
            for lane_yaml in level_yaml["lanes"].as_sequence().unwrap() {
                let data = lane_yaml.as_sequence().unwrap();
                let start = data[0].as_u64().unwrap();
                let end = data[1].as_u64().unwrap();
                let lane = Lane {
                    start: start as usize,
                    end: end as usize
                };
                self.lanes.push(lane);
            }
            let walls_yaml = level_yaml["walls"].as_sequence();
            if walls_yaml.is_some() {
                for wall_yaml in walls_yaml.unwrap() {
                    let data = wall_yaml.as_sequence().unwrap();
                    let start = data[0].as_u64().unwrap();
                    let end = data[1].as_u64().unwrap();
                    let wall = Wall {
                        start: start as usize,
                        end: end as usize
                    };
                    self.walls.push(wall);
                }
            }
        }
    }

    fn _print(&self) {
        println!("site name: [{}]", &self.site_name);
        println!("vertices:");
        for v in &self.vertices {
            println!("{} {} {}", v.name, v.x, v.y);
        }
    }

    fn spawn(
        &self,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        _asset_server: Res<AssetServer>,
    ) {
        let mut ofs_x = 0.0;
        let mut ofs_y = 0.0;
        let scale = 1.0 / 100.0;
        let mut num_v = 0;
        for v in &self.vertices {
            ofs_x += v.x;
            ofs_y += v.y;
            num_v += 1;
        }
        ofs_x /= num_v as f64;
        ofs_y /= num_v as f64;

        let vertex_handle = meshes.add(
            Mesh::from(
                shape::Capsule {
                    radius: 0.25,
                    rings: 2,
                    depth: 0.05,
                    latitudes: 8,
                    longitudes: 16,
                    uv_profile: shape::CapsuleUvProfile::Fixed,
                }
            )
        );

        for v in &self.vertices {
            commands.spawn_bundle(PbrBundle {
                mesh: vertex_handle.clone(),
                material: materials.add(Color::rgb(0.4, 0.7, 0.6).into()),
                transform: Transform {
                    translation: Vec3::new(
                        ((v.x - ofs_x) * scale) as f32,
                        ((v.y - ofs_y) * scale) as f32,
                        0.0,
                    ),
                    rotation: Quat::from_rotation_x(1.57),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        let mut z_ofs = 0.01;
        for lane in &self.lanes {
            let v1 = &self.vertices[lane.start];
            let v2 = &self.vertices[lane.end];
            let v1x = ((v1.x - ofs_x) * scale) as f32;
            let v1y = ((v1.y - ofs_y) * scale) as f32;
            let v2x = ((v2.x - ofs_x) * scale) as f32;
            let v2y = ((v2.y - ofs_y) * scale) as f32;

            let dx = v2x - v1x;
            let dy = v2y - v1y;
            let length = Vec2::from([dx, dy]).length();
            let width = 0.5 as f32;
            let yaw = dy.atan2(dx);
            let cx = (v1x + v2x) / 2.;
            let cy = (v1y + v2y) / 2.;

            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::from([length, width])))),
                material: materials.add(Color::rgba(1.0, 0.5, 0.3, 0.5).into()),
                transform: Transform {
                    translation: Vec3::new(cx, cy, z_ofs),
                    rotation: Quat::from_rotation_z(yaw),
                    ..Default::default()
                },
                ..Default::default()
            });
            z_ofs += 0.001;  // avoid flicker
        }

        for wall in &self.walls {
            let v1 = &self.vertices[wall.start];
            let v2 = &self.vertices[wall.end];
            let v1x = ((v1.x - ofs_x) * scale) as f32;
            let v1y = ((v1.y - ofs_y) * scale) as f32;
            let v2x = ((v2.x - ofs_x) * scale) as f32;
            let v2y = ((v2.y - ofs_y) * scale) as f32;

            let dx = v2x - v1x;
            let dy = v2y - v1y;
            let length = Vec2::from([dx, dy]).length();
            let width = 0.1 as f32;
            let height = 1.0 as f32;
            let yaw = dy.atan2(dx);
            let cx = (v1x + v2x) / 2.;
            let cy = (v1y + v2y) / 2.;

            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(length, width, height))),
                material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
                transform: Transform {
                    translation: Vec3::new(cx, cy, height / 2.),
                    rotation: Quat::from_rotation_z(yaw),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }
}

fn initialize_site_map(
    mut sm: ResMut<SiteMap>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        println!("parsing...");
        sm.load(args[1].clone());
        sm.spawn(commands, meshes, materials, asset_server);
        println!("parsing complete");
    } else {
        sm.load_demo();
        sm.spawn(commands, meshes, materials, asset_server);
    }
}

fn handle_keyboard(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut FlexibleProjection>,
) {
    let mut projection = query.single_mut();
    if keyboard_input.just_pressed(KeyCode::Key2) {
        projection.set_mode(ProjectionMode::Orthographic);
    }

    if keyboard_input.just_pressed(KeyCode::Key3) {
        projection.set_mode(ProjectionMode::Perspective);
    }
}

fn egui_ui(
    mut sm: ResMut<SiteMap>,
    egui_context: ResMut<EguiContext>,
    mut query: Query<&mut FlexibleProjection>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut projection = query.single_mut();
    egui::TopBottomPanel::top("top_panel")
        .show(egui_context.ctx(), |ui| {
            ui.vertical(|ui| {

                egui::menu::bar(ui, |ui| {
                    egui::menu::menu(ui, "File", |ui| {
                        if ui.button("Load demo").clicked() {
                            sm.load_demo();
                            sm.spawn(commands, meshes, materials, asset_server);
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        if ui.button("Quit").clicked() {
                            std::process::exit(0);
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("[toolbar buttons]");
                    ui.separator();
                    if ui.add(egui::SelectableLabel::new(projection.mode == ProjectionMode::Orthographic, "2D")).clicked() {
                        projection.set_mode(ProjectionMode::Orthographic);
                    }
                    if ui.add(egui::SelectableLabel::new(projection.mode == ProjectionMode::Perspective, "3D")).clicked() {
                        projection.set_mode(ProjectionMode::Perspective);
                    }
                });
            });
        });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("entering setup() startup system...");

    /*
    // this is useful for debugging lighting... spheres are very forgiving
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 20.,
            ..Default::default()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::LIME_GREEN,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0., 0., 0.),
        ..Default::default()
    });
    */

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        //material: materials.add(Color::rgb(0.3, 0.7, 0.3).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.3, 0.3),
            ..Default::default()
        }),
        transform: Transform {
            rotation: Quat::from_rotation_x(1.57),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.001,
    });

    const HALF_SIZE: f32 = 1.;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10. * HALF_SIZE,
                far: 10. * HALF_SIZE,
                ..Default::default()
            },
            color: Color::rgb(1., 1., 1.),
            illuminance: 10000.,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0., 0., 50.),
            rotation: Quat::from_rotation_x(0.4),
            ..Default::default()
        },
        ..Default::default()
    });
}

#[wasm_bindgen]
pub fn run() {

    #[cfg(target_arch = "wasm32")]
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Traffic Editor III".to_string(),
            width: 400.,
            //height: 720.,
            //canvas: "te3_canvas".to_string(),
            canvas: Some(String::from("#te3_canvas")),
            //mode: WindowMode::BorderlessFullscreen,
            //vsync: false,
            ..Default::default()
        })
        .add_plugins(PipelinedDefaultPlugins)
        .insert_resource( DirectionalLightShadowMap {
            size: 2048
        })
        .init_resource::<SiteMap>()
        .add_plugin(SuperCameraPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(initialize_site_map.system())
        .add_system(handle_keyboard.system())
        .add_plugin(EguiPlugin)
        .add_system(egui_ui.system())
        .run();

    #[cfg(not(target_arch = "wasm32"))]
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Traffic Editor III".to_string(),
            width: 800.,
            height: 800.,
            //vsync: false,
            ..Default::default()
        })
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        //.insert_resource(Msaa { samples: 4})
        .init_resource::<SiteMap>()
        .add_plugin(SuperCameraPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(initialize_site_map.system())
        .add_system(handle_keyboard.system())
        .add_plugin(EguiPlugin)
        .add_system(egui_ui.system())
        .run();
}
