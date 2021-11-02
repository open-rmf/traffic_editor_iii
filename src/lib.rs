use bevy::{
    core::{FixedTimestep, Time},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::prelude::*,
    math::{Quat, Vec3},
    prelude::{App, Assets, AssetServer, Transform},
    pbr2::{PbrBundle, AmbientLight, /*PointLightBundle, PointLight,*/ StandardMaterial, DirectionalLightBundle, DirectionalLight},
    PipelinedDefaultPlugins,
    input::keyboard::KeyCode,
    input::Input,
    render2::{
        camera::OrthographicProjection,
        color::Color,
        mesh::{shape, Mesh},
        //view::Msaa
    },
    window::{WindowDescriptor, Windows},
};
use wasm_bindgen::prelude::*;

extern crate web_sys;
mod demo_world;

use bevy_egui::{egui, EguiContext, EguiPlugin};

mod supercamera;
use supercamera::{SuperCameraPlugin, FlexibleProjection, ProjectionMode};

mod site_map;
use site_map::{initialize_site_map, SiteMap};


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

#[cfg(target_arch = "wasm32")]
fn check_browser_window_size(mut windows: ResMut<Windows>) {
    let mut window = windows.get_primary_mut().unwrap();
    let wasm_window = web_sys::window().unwrap();
    let target_width = wasm_window.inner_width().unwrap().as_f64().unwrap() as f32;
    let target_height = wasm_window.inner_height().unwrap().as_f64().unwrap() as f32;
    let w_diff = (target_width - window.width()).abs();
    let h_diff = (target_height - window.height()).abs();
    // web_sys::console::log_1(&format!("diffs: {} {}", w_diff, h_diff).into());

    if w_diff > 3. || h_diff > 3. {
        web_sys::console::log_1(&format!("window = {} {} canvas = {} {}", window.width(), window.height(), target_width, target_height).into());
        window.set_resolution(target_width, target_height);
    }
}

#[wasm_bindgen]
pub fn run() {

    #[cfg(target_arch = "wasm32")]
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Traffic Editor III".to_string(),
            canvas: Some(String::from("#te3_canvas")),
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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.5))
                .with_system(check_browser_window_size.system())
            )
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
