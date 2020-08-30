use bevy::{
    input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
    render::camera::PerspectiveProjection,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pass::ClearColor,
    render::pipeline::PrimitiveTopology,
    window::CursorMoved,
};

#[derive(Default)]
struct State {
    // Collects mouse motion in the form of an x/y delta Vec2
    mouse_motion_event_reader: EventReader<MouseMotion>,
    // Collects mouse scroll motion in x/y
    mouse_wheel_event_reader: EventReader<MouseWheel>,
    // Collects cursor position on screen in x/y
    cursor_moved_event_reader: EventReader<CursorMoved>,
    window_info_event_reader: EventReader<WindowDescriptor>,
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.8, 0.8, 0.8)))
        .add_resource(Msaa { samples: 4 })
        .init_resource::<State>()
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(process_user_input.system())
        .add_system(update_camera.system())
        .add_system(cursor_pick.system())
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

    //dbg!(light_entity);

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
        /*.spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 10,
            })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 5,
            })),
            material: geometry_material_handle.clone(),
            translation: Translation::new(0.0, 3.0, 8.0),
            ..Default::default()
        })*/
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
    let l_mouse: bool = mouse_button_inputs.pressed(MouseButton::Left);
    let m_mouse: bool = mouse_button_inputs.pressed(MouseButton::Middle);
    let r_mouse: bool = mouse_button_inputs.pressed(MouseButton::Right);

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
            Some(CameraManipulation::Pan(mouse_move)) => {}
            Some(CameraManipulation::Rotate(mouse_move)) => {}
        }
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
                    transform.sync = false;
                }
            }
        }
    }
}

fn cursor_pick(
    // Resources
    mut state: ResMut<State>,
    cursor: Res<Events<CursorMoved>>,
    meshes: Res<Assets<Mesh>>,
    windows: Res<Windows>,
    // Components
    mut query: Query<(&Handle<Mesh>, &Transform)>,
    mut orbit_camera_query: Query<&OrbitCamera>,
    transform_query: Query<(&Transform, &PerspectiveProjection)>,
) {
    // Get the cursor position
    let mut cursor_position = Vec2::zero();
    match state.cursor_moved_event_reader.latest(&cursor) {
        Some(cursor_moved) => cursor_position = cursor_moved.position,
        None => return,
    }
    // Get current screen size
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2::from([window.width as f32, window.height as f32]);
    // Normalized device coordinates (NDC) describes cursor position from (-1, -1) to (1, 1)
    let ndc_cursor: Vec2 = (cursor_position / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    // Create near and far 3d positions using the mouse coordinates
    let far_point_ndc = Vec3::new(ndc_cursor[0], ndc_cursor[1], 1.0);
    let far_point_camera_ndc = Vec3::new(0.0, 0.0, 1.0);

    // projection * camera.inverse * mesh transform
    // compare xy with cursor xy

    // Get the view transform from the camera, 
    let mut view_matrix = Mat4::zero();
    let mut projection_matrix = Mat4::zero();
    for orbit_camera in &mut orbit_camera_query.iter() {
        if let Some(camera_entity) = orbit_camera.cam_entity {
            if let Ok(transform) = transform_query.get::<Transform>(camera_entity) {
                view_matrix = transform.value.inverse();
            }
            if let Ok(proj) = transform_query.get::<PerspectiveProjection>(camera_entity) {
                projection_matrix =
                    Mat4::perspective_rh(proj.fov, proj.aspect_ratio, proj.near, proj.far);
            }
        }
    }

    // Iterate through each mesh in the scene
    for (mesh_handle, transform) in &mut query.iter() {
        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get(mesh_handle) {
            if mesh.primitive_topology != PrimitiveTopology::TriangleList {
                break;
            }
            // we need to move the mesh from model space to world space using its transform,
            // them move it with the inverse of the cursor ray transform, to place it in a
            // coordinate space relative to the cursor vector
            let combined_transform = projection_matrix * view_matrix * transform.value;
            let mut vertices = Vec::new();
            for attribute in mesh.attributes.iter() {
                if attribute.name != VertexAttribute::POSITION {
                    break;
                }
                match &attribute.values {
                    VertexAttributeValues::Float3(positions) => vertices = positions.clone(),
                    _ => {}
                }
            }

            if let Some(indices) = &mesh.indices {
                // Now that we're in the vector of vertex indices, we want to look at the vertex
                // positions for each triangle, so we'll take indices in chunks of three, where each
                // chunk of three indices defines the three vertices of a triangle.
                for index in indices.chunks(3) {
                    // With the three vertex positions of the current triangle available,
                    // we need to transform the 3d positions from the mesh's space, to the world
                    // space using the mesh's transform, then move it relative to the camera's
                    // space using the view matrix (camera.inverse), and finally apply the
                    // perspective matrix. The position of each vertex should now be given to us
                    // relative to the NDC space.
                    let v = Vec3::zero();
                    let mut triangle: [Vec3; 3] = [v, v, v];
                    // Make sure this chunk has 3 vertices to avoid a panic.
                    if index.len() == 3 {
                        for i in 0..3 {
                            triangle[i] = combined_transform
                                .transform_point3(Vec3::from(vertices[index[i] as usize]));
                        }
                    }
                    if point_in_tri(
                        &ndc_cursor,
                        &Vec2::new(triangle[0].x(), triangle[0].y()),
                        &Vec2::new(triangle[1].x(), triangle[1].y()),
                        &Vec2::new(triangle[2].x(), triangle[2].y()),
                    ) {
                        println!("HIT! {}\n{}\n{:?}", mesh_handle.id.0, ndc_cursor, triangle);
                        break;
                    } else {
                        //println!("{}\n{:?}", far_point_world, triangle);
                    }
                }
            }
        }
    }
}

fn double_tri_area(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    f32::abs(a.x() * (b.y() - c.y()) + b.x() * (c.y() - a.y()) + c.x() * (a.y() - b.y()))
}

fn point_in_tri(p: &Vec2, a: &Vec2, b: &Vec2, c: &Vec2) -> bool {
    let area = double_tri_area(a, b, c);
    let pab = double_tri_area(p, a, b);
    let pac = double_tri_area(p, a, c);
    let pbc = double_tri_area(p, b, c);
    let area_tris = pab + pac + pbc;
    let epsilon = 0.000000001;
    //println!("{:.3}  {:.3}", area, area_tris);
    f32::abs(area - area_tris) < epsilon
}