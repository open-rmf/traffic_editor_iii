use bevy::{
    core_pipeline::ClearColor,
    ecs::bundle::Bundle,
    ecs::prelude::*,
    input::{
        Input,
        mouse::{MouseButton, MouseWheel},
    },
    math::{Vec2, Vec3, Mat3, Mat4, Quat},
    prelude::{App, Assets, CoreStage, GlobalTransform, Plugin, Transform},
    pbr2::{StandardMaterial},
    reflect::Reflect,
    render2::mesh::Mesh,
    render2::{
        camera::{
            camera_system,
            Camera,
            CameraPlugin,
            CameraProjection,
            DepthCalculation,
            PerspectiveProjection,
            OrthographicProjection,
            ScalingMode,
            WindowOrigin,
        },
        color::Color,
    },
    window::{CursorMoved, Windows},
};
use bevy::ecs as bevy_ecs;
use bevy::reflect as bevy_reflect;

extern crate web_sys;

#[derive(PartialEq, Debug, Clone, Reflect)]
pub enum ProjectionMode { Perspective, Orthographic }

#[derive(Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FlexibleProjection {
    pub mode: ProjectionMode,
    mode_switched: bool,
    perspective: PerspectiveProjection,
    orthographic: OrthographicProjection,
    orbit_center: Vec3,
    orbit_radius: f32,
    orbit_upside_down: bool,
}

impl Default for FlexibleProjection {
    fn default() -> Self {
        FlexibleProjection {
            mode: ProjectionMode::Orthographic,
            mode_switched: true,
            perspective: Default::default(),
            orthographic: OrthographicProjection {
                window_origin: WindowOrigin::Center,
                scaling_mode: ScalingMode::FixedVertical,
                scale: 10.0,
                ..Default::default()
            },
            orbit_center: Vec3::ZERO,
            orbit_radius: 100.0,
            orbit_upside_down: false,
        }
    }
}

impl CameraProjection for FlexibleProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        if self.mode == ProjectionMode::Perspective {
            return self.perspective.get_projection_matrix();
        } else {
            return self.orthographic.get_projection_matrix();
        }
    }

    fn update(&mut self, width: f32, height: f32) {
        self.perspective.update(width, height);
        self.orthographic.update(width, height);
    }

    fn depth_calculation(&self) -> DepthCalculation {
        if self.mode == ProjectionMode::Perspective {
            return self.perspective.depth_calculation();
        } else {
            return self.orthographic.depth_calculation();
        }
    }
}

impl FlexibleProjection {
    pub fn set_mode(&mut self, mode: ProjectionMode) {
        self.mode = mode;
        self.mode_switched = true;
    }
}

#[derive(Bundle)]
pub struct SuperCameraBundle {
    pub camera: Camera,
    pub flexible_projection: FlexibleProjection,
    pub initial_position: Vec3,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for SuperCameraBundle {
    fn default() -> Self {
        SuperCameraBundle {
            camera: Camera {
                name: Some(CameraPlugin::CAMERA_3D.to_string()),
                ..Default::default()
            },
            flexible_projection: Default::default(),
            initial_position: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}

struct MouseLocation {
    previous: Vec2,
}

impl Default for MouseLocation {
    fn default() -> Self {
        MouseLocation {
            previous: Vec2::ZERO,
        }
    }
}

fn supercamera_motion(
    windows: Res<Windows>,
    mut ev_cursor_moved: EventReader<CursorMoved>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut previous_mouse_location: ResMut<MouseLocation>,
    mut query: Query<(&mut Camera, &mut Transform, &mut FlexibleProjection, &Vec3)>,
) {
    let pan_button = MouseButton::Left;
    let orbit_button = MouseButton::Right;

    // spin through all mouse cursor-moved events to find the last one
    let mut last_pos = previous_mouse_location.previous;
    for ev in ev_cursor_moved.iter() {
        last_pos.x = ev.position.x;
        last_pos.y = ev.position.y;
    }

    let mut cursor_motion = Vec2::ZERO;
    if input_mouse.pressed(pan_button) || input_mouse.pressed(orbit_button) {
        cursor_motion.x = last_pos.x - previous_mouse_location.previous.x;
        cursor_motion.y = last_pos.y - previous_mouse_location.previous.y;
    }

    previous_mouse_location.previous = last_pos;

    let mut scroll = 0.0;
    for ev in ev_scroll.iter() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            scroll += ev.y;
        }
        #[cfg(target_arch = "wasm32")]
        {
            // scrolling in wasm is a different beast
            scroll += 4. * ev.y / ev.y.abs();
        }
    }

    /*
    if scroll.abs() > 0.0 {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("scroll = {}", scroll).into());
        //println!("scroll = {}", scroll);
    }
    */

    #[cfg(target_arch = "wasm32")]
    {
      scroll = scroll * 0.1; // not sure why, but web scrolling seems SO FAST
    }

    let (_camera, mut transform, mut projection, initial_position) = query.single_mut();

    if projection.mode_switched {
        projection.mode_switched = false;
        transform.translation = initial_position.clone();
        transform.rotation = Quat::default();
        /*
        if projection.mode == ProjectionMode::Perspective {
            transform.translation.z = projection.orbit_radius;
            println!("set transform translation to {}", transform.translation);
        }
        */
    }

    if projection.mode == ProjectionMode::Orthographic {
        if cursor_motion.length_squared() > 0.0 {
            let window = windows.get_primary().unwrap();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let aspect_ratio = window_size[0] / window_size[1];
            cursor_motion *= 2. / window_size * Vec2::new(
                projection.orthographic.scale * aspect_ratio,
                projection.orthographic.scale
            );
            let right = -cursor_motion.x * Vec3::X;
            let up = -cursor_motion.y * Vec3::Y;
            transform.translation += right + up;
        }
        if scroll.abs() > 0.0 {
            projection.orthographic.scale -= scroll * projection.orthographic.scale * 0.1;
            projection.orthographic.scale = f32::max(projection.orthographic.scale, 0.02);
        }
    } else {
        // perspective mode
        if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Z;
            projection.orbit_upside_down = up.z <= 0.0;
        }

        let mut any = false;
        if input_mouse.pressed(orbit_button) && cursor_motion.length_squared() > 0. {
            any = true;
            let window = windows.get_primary().unwrap();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let delta_x = {
                let delta = cursor_motion.x / window_size.x * std::f32::consts::PI * 2.0;
                if projection.orbit_upside_down { -delta } else { delta }
            };
            let delta_y = -cursor_motion.y / window_size.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_z(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // global y
            transform.rotation = transform.rotation * pitch; // local x
        } else if input_mouse.pressed(MouseButton::Left) && cursor_motion.length_squared() > 0. {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = windows.get_primary().unwrap();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);

            cursor_motion *=
                Vec2::new(
                    projection.perspective.fov * projection.perspective.aspect_ratio,
                    projection.perspective.fov
                ) / window_size;
            // translate by local axes
            let right = transform.rotation * Vec3::X * -cursor_motion.x;
            let up = transform.rotation * Vec3::Y * -cursor_motion.y;
            // make panning proportional to distance away from center point
            let translation = (right + up) * projection.orbit_radius;
            projection.orbit_center += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            projection.orbit_radius -= scroll * projection.orbit_radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            projection.orbit_radius = f32::max(projection.orbit_radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                projection.orbit_center
                + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, projection.orbit_radius));
        }
    }
}

fn supercamera_setup(
    mut commands: Commands,
    mut _meshes: ResMut<Assets<Mesh>>,
    mut _materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("supercamera_setup()");
    let mut cam = SuperCameraBundle::default();
    // todo: calculate camera scale based on window size and map size, etc.
    cam.flexible_projection.orthographic.scale = 100.0;
    let start_z = 200.;
    cam.flexible_projection.orbit_radius = start_z;
    cam.initial_position = Vec3::new(0., 0., start_z);
    cam.transform = Transform::from_translation(cam.initial_position).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(cam);
}

#[derive(Default)]
pub struct SuperCameraPlugin;

impl Plugin for SuperCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseLocation>()
           .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
           .add_startup_system(supercamera_setup.system())
           .add_system(supercamera_motion);

        app.register_type::<Camera>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                camera_system::<FlexibleProjection>,
            );
    }
}
