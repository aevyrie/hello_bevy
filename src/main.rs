use bevy:: {
    prelude::*,
    input::mouse::{
        MouseMotion,
        MouseWheel,
        MouseButton,
    },
    render::pass::ClearColor,
};

#[derive(Default)]
struct State {
    // Collects mouse motion in the form of an x/y delta Vec2
    mouse_motion_event_reader: EventReader<MouseMotion>,
    // Collects mouse scroll motion in x/y
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_resource(Msaa { samples: 4 })
        .add_resource(ElapsedTime(Timer::from_seconds(2.0)))
        .init_resource::<State>()
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(process_mouse_events.system())
        .add_system(update_camera.system())
        .run();
}

struct ElapsedTime(Timer);

struct OrbitCamera {
    cam_distance: f32,
    cam_pitch: f32,
    cam_yaw: f32,
    cam_entity: Option<Entity>,
    light_entity: Option<Entity>,
}


impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            cam_distance: 20.,
            cam_pitch: 30.0f32.to_radians(),
            cam_yaw: 0.0,
            cam_entity: None,
            light_entity: None,
        }
    }
}

struct LightIndicator{}

/// Perform scene creation, creating meshes, cameras, and lights
fn setup(
    // Commands
    mut commands: Commands,
    // Resources
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // Set up the geometry material
    let geometry_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(1.0, 1.0, 1.0),
        shaded: true,
        ..Default::default()
    });

    let rotation_center_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(1.0, 0.0, 0.0),
        shaded: false,
        ..Default::default()
    });


    let cam_entity = commands.spawn(Camera3dComponents::default()).current_entity();

    let light_entity = commands.spawn(LightComponents{
        translation: Translation::new(5.0, 5.0, 5.0),
        light: Light{color: Color::rgb(0.0, 0.0, 1.0),..Default::default()},
        ..Default::default()
    }).current_entity();

    //dbg!(light_entity);

    let rotation_center_entity = commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.1, subdivisions: 1 })),
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
            &[cam_entity.unwrap(), light_entity.unwrap()])
        // Add some geometry
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(-2.0, -2.0, -2.0),
            ..Default::default()
        })
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 1.0, subdivisions: 5 })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 1.0, subdivisions: 5 })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(0.0, 3.0, 8.0),
            ..Default::default()
        })
        .with(LightIndicator{})
        // Create the environment.
        .spawn(LightComponents {
            translation: Translation::new(5.0, 5.0, 5.0),
            ..Default::default()
        });
}

/// Process user mouse input and update the camera
fn process_mouse_events(
    // Resources
    time: Res<Time>,
    mut state: ResMut<State>, 
    mouse_button_inputs: Res<Input<MouseButton>>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mouse_wheel_events: Res<Events<MouseWheel>>,
    // Component Queries
    mut query: Query<&mut OrbitCamera>,
) {
    // Get the mouse movement since the last frame
    let mut mouse_movement = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        mouse_movement = event.delta;
    }
    // Get the scroll wheel movement since the last frame
    let mut scroll_amount = 0.0;
    for event in state.mouse_wheel_event_reader.iter(&mouse_wheel_events) {
        scroll_amount = event.y as f32;
    }
    // Scaling factors for zooming and rotation
    let zoom_scale = 50.0;
    let look_scale = 1.0;

    for mut camera in &mut query.iter() {
        if mouse_button_inputs.pressed(MouseButton::Middle) {
            camera.cam_yaw += mouse_movement.x() * time.delta_seconds;
            camera.cam_pitch -= mouse_movement.y() * time.delta_seconds * look_scale;
        }
        camera.cam_distance -= scroll_amount * time.delta_seconds * zoom_scale;
    }
}

fn update_camera(
    // Resources
    time: Res<Time>,
    mut timer: ResMut<ElapsedTime>,
    // Component Queries
    mut rotation_center_query: Query<(&mut OrbitCamera, &mut Rotation)>,
    camera_query: Query<(&mut Translation, &mut Rotation, &mut Transform)>,
    light_query: Query<(&mut Translation, &mut Light, &mut Transform)>,
) {
    if timer.0.finished {timer.0.reset()};
    timer.0.tick(time.delta_seconds);

    for (mut center,  mut rotation) in &mut rotation_center_query.iter() {
        center.cam_pitch = center.cam_pitch.max(1f32.to_radians()).min(179f32.to_radians());
        center.cam_distance = center.cam_distance.max(5.).min(30.);

        rotation.0 = Quat::from_rotation_y(-center.cam_yaw);

        if let Some(camera_entity) = center.cam_entity {
            let cam_pos = Vec3::new(
                    0.0, 
                    center.cam_pitch.cos(), 
                    -center.cam_pitch.sin()
                ).normalize()* center.cam_distance;

            if let Ok(mut translation) = camera_query.get_mut::<Translation>(camera_entity) {
                translation.0 = cam_pos;
            }

            if let Ok(mut rotation) = camera_query.get_mut::<Rotation>(camera_entity) {
                let look = Mat4::face_toward(cam_pos, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
                rotation.0 = look.to_scale_rotation_translation().1;
            }

            let mut camera_transform = Mat4::default();

            if let Ok(mut transform) = camera_query.get_mut::<Transform>(camera_entity) {
                camera_transform = transform.value;
            }
        
            if let Some(light_entity) = center.light_entity {
                let light_pos = Vec3::new(
                    5.0 * timer.0.elapsed.mul_add(6.28, 0.0).sin(), 
                    5.0 * timer.0.elapsed.mul_add(3.14, 0.0).sin(), 
                    5.0 * timer.0.elapsed.mul_add(3.14, 0.0).cos(), 
                ).normalize() * center.cam_distance;

                if let Ok(mut translation) = light_query.get_mut::<Translation>(light_entity) {
                    //translation.0 = Translation::from()
                    //dbg!(translation, timer.0.elapsed);
                }

                if let Ok(mut transform) = light_query.get_mut::<Transform>(light_entity) {
                    transform.value = camera_transform;
                    transform.sync = true;
                }
            }
        }
    }
}