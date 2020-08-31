use bevy::{
    input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    render::pass::ClearColor,
};
use bevy_mod_picking::*;

#[derive(Default)]
struct State {
    // Collects mouse motion in the form of an x/y delta Vec2
    mouse_motion_event_reader: EventReader<MouseMotion>,
    // Collects mouse scroll motion in x/y
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_resource(Msaa { samples: 4 })
        .init_resource::<State>()
        .add_default_plugins()
        .add_plugin(ModPicking)
        .add_startup_system(setup.system())
        .add_system(process_user_input.system())
        .add_system(update_camera.system())
        //.add_system(cursor_pick.system())
        .run();
}

struct OrbitCamera {
    cam_distance: f32,
    cam_pitch: f32,
    cam_yaw: f32,
    cam_entity: Option<Entity>,
    light_entity: Option<Entity>,
    camera_manipulation: Option<CameraManipulation>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            cam_distance: 20.,
            cam_pitch: 30.0f32.to_radians(),
            cam_yaw: 0.0,
            cam_entity: None,
            light_entity: None,
            camera_manipulation: None,
        }
    }
}

struct LightIndicator {}

/// Perform scene creation, creating meshes, cameras, and lights
fn setup(
    // Commands
    mut commands: Commands,
    // Resources
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pick_state: ResMut<MousePicking>,
) {
    // Set up the geometry material
    let geometry_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(1.0, 1.0, 1.0),
        shaded: true,
        ..Default::default()
    });

    pick_state.hovered_material = materials.add(StandardMaterial {
        albedo: Color::rgb(0.5, 0.5, 1.0),
        shaded: true,
        ..Default::default()
    });

    pick_state.selected_material = materials.add(StandardMaterial {
        albedo: Color::rgb(0.5, 1.0, 0.5),
        shaded: true,
        ..Default::default()
    });

    let rotation_center_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(1.0, 0.0, 0.0),
        shaded: false,
        ..Default::default()
    });

    let cam_entity = commands
        .spawn(Camera3dComponents::default())
        .current_entity();

    let light_entity = commands
        .spawn(LightComponents {
            translation: Translation::new(0.0, 0.0, 5.0),
            light: Light {
                color: Color::rgb(0.5, 0.5, 0.5),
                ..Default::default()
            },
            ..Default::default()
        })
        .current_entity();

    let rotation_center_entity = commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                subdivisions: 1,
            })),
            material: rotation_center_material_handle.clone(),
            translation: Translation::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .with(OrbitCamera {
            cam_entity,
            light_entity,
            cam_distance: 20.0,
            ..Default::default()
        })
        .current_entity();

    commands
        // Append camera to rotation center as child.
        .push_children(
            rotation_center_entity.unwrap(),
            &[cam_entity.unwrap(), light_entity.unwrap()],
        )
        // Add some geometry
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(-2.0, -2.0, -2.0),
            ..Default::default()
        })
        .with(Selectable::default())
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 10,
            })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(3.0, -0.0, 0.0),
            ..Default::default()
        })
        .with(Selectable::default())
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 5,
            })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(0.0, 3.0, 8.0),
            ..Default::default()
        })
        .with(Selectable::default())
        .with(LightIndicator {})
        // Create the environment.
        .spawn(LightComponents {
            translation: Translation::new(30.0, 100.0, 30.0),
            light: Light {
                color: Color::rgb(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        });
}

#[derive(Clone)]
enum CameraManipulation {
    Pan(MouseMotion),
    Orbit(MouseMotion),
    Rotate(MouseMotion),
    Zoom(MouseWheel),
}

/// Process user input and determine needed output
fn process_user_input(
    // Resources
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    keyboard_input: Res<Input<KeyCode>>,
    // Component Queries
    mut query: Query<&mut OrbitCamera>,
) {
    // Get the mouse movement since the last frame
    let mut mouse_movement = MouseMotion {
        delta: Vec2::new(0.0, 0.0),
    };
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        mouse_movement = event.clone();
    }
    // Get the scroll wheel movement since the last frame
    let mut scroll_amount = MouseWheel {
        unit: MouseScrollUnit::Pixel,
        x: 0.0,
        y: 0.0,
    };
    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        scroll_amount = event.clone();
    }
    // Scaling factors for zooming and rotation
    let zoom_scale = 50.0;
    let look_scale = 1.0;

    let l_alt: bool = keyboard_input.pressed(KeyCode::LAlt);
    let l_shift: bool = keyboard_input.pressed(KeyCode::LShift);
    //let l_mouse: bool = mouse_button_inputs.pressed(MouseButton::Left);
    let m_mouse: bool = mouse_button_inputs.pressed(MouseButton::Middle);
    //let r_mouse: bool = mouse_button_inputs.pressed(MouseButton::Right);

    let manipulation = if l_alt && m_mouse {
        Some(CameraManipulation::Pan(mouse_movement))
    } else if l_shift && m_mouse {
        Some(CameraManipulation::Rotate(mouse_movement))
    } else if m_mouse {
        Some(CameraManipulation::Orbit(mouse_movement))
    } else if scroll_amount.y != 0.0 {
        Some(CameraManipulation::Zoom(scroll_amount))
    } else {
        None
    };

    for mut camera in &mut query.iter() {
        match &manipulation {
            None => {}
            Some(CameraManipulation::Orbit(mouse_move)) => {
                camera.cam_yaw += mouse_move.delta.x() * time.delta_seconds;
                camera.cam_pitch -= mouse_move.delta.y() * time.delta_seconds * look_scale;
            }
            Some(CameraManipulation::Zoom(scroll)) => {
                camera.cam_distance -= scroll.y * time.delta_seconds * zoom_scale;
            }
            Some(CameraManipulation::Pan(_)) => {}
            Some(CameraManipulation::Rotate(_)) => {}
        }
        camera.camera_manipulation = manipulation.clone();
    }
}

fn update_camera(
    // Resources
    // Component Queries
    mut rotation_center_query: Query<(&mut OrbitCamera, &mut Rotation)>,
    camera_query: Query<(&mut Translation, &mut Rotation, &mut Transform)>,
    light_query: Query<(&mut Translation, &mut Light, &mut Transform)>,
) {
    // Take the results of the orbit cam query
    for (mut orbit_center, mut rotation) in &mut rotation_center_query.iter() {
        orbit_center.cam_pitch = orbit_center
            .cam_pitch
            .max(1f32.to_radians())
            .min(179f32.to_radians());
        orbit_center.cam_distance = orbit_center.cam_distance.max(5.).min(30.);

        rotation.0 = Quat::from_rotation_y(-orbit_center.cam_yaw);

        //  If a camera entity exists in the query
        if let Some(camera_entity) = orbit_center.cam_entity {
            let cam_pos = Vec3::new(
                0.0,
                orbit_center.cam_pitch.cos(),
                -orbit_center.cam_pitch.sin(),
            )
            .normalize()
                * orbit_center.cam_distance;

            if let Ok(mut translation) = camera_query.get_mut::<Translation>(camera_entity) {
                translation.0 = cam_pos;
            }

            if let Ok(mut rotation) = camera_query.get_mut::<Rotation>(camera_entity) {
                let look = Mat4::face_toward(cam_pos, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
                rotation.0 = look.to_scale_rotation_translation().1;
            }

            let mut camera_transform = Mat4::default();

            if let Ok(transform) = camera_query.get_mut::<Transform>(camera_entity) {
                camera_transform = transform.value;
            }

            if let Some(light_entity) = orbit_center.light_entity {
                if let Ok(mut translation) = light_query.get_mut::<Translation>(light_entity) {
                    // get the quat the corresponds to the current yaw of the camera
                    let light_rot = Quat::from_rotation_y(-orbit_center.cam_yaw);
                    //
                    translation.0 = light_rot.mul_vec3(cam_pos.into());
                }

                if let Ok(mut transform) = light_query.get_mut::<Transform>(light_entity) {
                    transform.value = camera_transform;
                    //transform.sync = false;
                }
            }
        }
    }
}