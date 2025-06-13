use bevy::{color::palettes::css::{BLUE}, prelude::*, pbr::CascadeShadowConfigBuilder};
use bevy_rapier3d::{prelude::*};
use std::f32::consts::PI;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy::log::LogPlugin;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: "info".into(),
            level: bevy::log::Level::INFO,
            custom_layer: |_| None,

        }),
        RapierPhysicsPlugin::<NoUserData>::default(),
        PanOrbitCameraPlugin,
        RapierDebugRenderPlugin::default()
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, (
        tank_movement,
        spawn_projectile,
       // move_projectile,
        camera_follow,
        elevate_canon,
        rotate_turret,
        tag_turret_after_spawn, 
    ))
    .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {

    // Light
    // commands.spawn((
    //     PointLight {
    //         intensity: 4000.0,
    //         shadows_enabled: true,
    //         color: WHITE.into(),
    //         ..default()},
    //     Transform::from_xyz(0.0, 20.0, 0.0)));

        commands.spawn((
            DirectionalLight {
                illuminance: light_consts::lux::OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-PI / 4.),
                ..default()
            },
            // The default cascade config is designed to handle large scenes.
            // As this example has a much smaller world, we can tighten the shadow
            // bounds for better visual quality.
            CascadeShadowConfigBuilder {
                first_cascade_far_bound: 4.0,
                maximum_distance: 10.0,
                ..default()
            }
            .build(),
        ));

    //Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(20.0,20.0).subdivisions(10)))),
        MeshMaterial3d(materials.add(StandardMaterial {base_color_texture: Some(asset_server.load("coast_sand_rocks_02_2k/textures/coast_sand_rocks_02_diff_2k.jpg")), ..Default::default()})),
        Collider::cuboid(10.0, 0.1, 10.0)
    ));

    //Tank
    commands.spawn((
        Transform::from_xyz(0.0, 0.5, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(0.64,0.4,0.8),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("tankBlend.glb"))),
        Tank
    ));

    //Pushable cylinder
    for x in -2..=2 {
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::default())),
            MeshMaterial3d(materials.add(StandardMaterial {base_color: BLUE.into(), ..Default::default()})),
            Transform::from_xyz(x as f32 * 2.0, 0.5, -4.0).with_rotation(Quat::from_rotation_x(90.0_f32.to_radians())),
            RigidBody::Dynamic,
            Collider::cylinder(0.5, 0.5)
        ));
    }

    //Camera
    commands.spawn((
        Transform::from_xyz(-15.0, 15.0, -15.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            enabled: false,
            ..default()
        },
        Camera::default(),
    ));

}

#[derive(Component)]
struct TankTurret;

#[derive(Component)]
struct TankCanon;

fn tag_turret_after_spawn(
    mut commands: Commands,
    query: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in query.iter() {
        if name.as_str().to_lowercase().contains("tankfree_canon") {
            commands.entity(entity).insert(TankCanon);
        }
        else if name.as_str().to_lowercase().contains("tankfree_tower") {
            commands.entity(entity).insert(TankTurret);
        }
    }
}

fn elevate_canon(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<TankCanon>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let rotation_speed = 1.5;

        if keys.pressed(KeyCode::ArrowUp) {
            transform.rotate_local_x(-rotation_speed * time.delta_secs());
        }
        if keys.pressed(KeyCode::ArrowDown) {
            transform.rotate_local_x(rotation_speed * time.delta_secs());
        }
    }
}

fn rotate_turret(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<TankTurret>>,
    time: Res<Time>,
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let rotation_speed = 1.5;

        if keys.pressed(KeyCode::ArrowLeft) {
            transform.rotate_local_y(-rotation_speed * time.delta_secs());
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.rotate_local_y(rotation_speed * time.delta_secs());
        }
    }
}

#[derive(Component)]
struct Tank;

fn tank_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Tank>>
)  {
    let mut transform = query.single_mut();
    let mut direction = 0.0;
    let mut rotation = 0.0;
    let drive_velocity = 0.8;
    let rotation_velocity = 0.5;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += drive_velocity;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        direction -= drive_velocity;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        rotation += rotation_velocity;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        rotation -= rotation_velocity;
    }

    let rotation_speed = 2.0;
    let move_speed = 5.0;

    transform.rotate_y(rotation * rotation_speed * time.delta_secs());
    let forward = transform.forward();
    transform.translation += forward * direction * move_speed * time.delta_secs();
}

#[derive(Component)]
struct Camera {
    use_follow_cam: bool,
}

impl Camera {
    fn default() -> Self {
        Self {
            use_follow_cam: true,
        }
    }
}

fn camera_follow(
    tank_query: Query<&Transform, (With<Tank>, Without<Camera>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Camera, &mut PanOrbitCamera)>
)   {

    for (mut camera, mut orbit_camera) in &mut query
    {
        if mouse.just_pressed(MouseButton::Middle) || camera.use_follow_cam {

            let tank_transform = tank_query.single();

            let focus = tank_transform.translation;
        
            let direction = (focus - tank_transform.forward() * 15.0 + Vec3::Y * 10.0) - focus;

            orbit_camera.target_focus = focus;
            orbit_camera.target_radius = direction.length();
            orbit_camera.target_yaw = direction.x.atan2(direction.z);
            orbit_camera.target_pitch = (direction.y / direction.length()).asin();

            if mouse.just_pressed(MouseButton::Middle) {
                camera.use_follow_cam = !camera.use_follow_cam;
                orbit_camera.enabled = !camera.use_follow_cam;
            }

            if !camera.use_follow_cam {
                orbit_camera.enabled = true;
           }
        }      
    }
}

#[derive(Component)]
struct Projectile;

fn spawn_projectile(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    tank_cannon_transform_query: Query<&GlobalTransform, With<TankCanon>>,
    asset_server: Res<AssetServer>
){
    if keyboard.just_pressed(KeyCode::Space) {
        if let Ok(tank_cannon_transform) = tank_cannon_transform_query.get_single() {

            let rotation = tank_cannon_transform.rotation() * Quat::from_rotation_y(std::f32::consts::PI);
            let direction = -tank_cannon_transform.forward();

            let launch_speed = 15.0;
            let upward_arc = 5.0;
            
            let velocity = direction * launch_speed + upward_arc;

            commands.spawn((
                Transform::from_translation(tank_cannon_transform.translation() + (direction * 0.7)).with_rotation(rotation),
                Projectile,
                SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("shell.glb"))),
                Collider::cuboid(0.05, 0.05, 0.05),
                RigidBody::Dynamic,
                Velocity::linear(velocity),
            ));
        }
    }
}

    
fn move_projectile(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Projectile>>
) {
    let projectile_velocity = 6.0;

    for mut transform in query.iter_mut() {
        let forward = transform.forward(); // This gives you the +Z direction
        transform.translation += forward * projectile_velocity * time.delta_secs();
    }
}