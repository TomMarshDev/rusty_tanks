use bevy::{color::palettes::css::{GREEN, BLUE}, prelude::*};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default()
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, tank_movement)
    //.add_systems(Update, camera_follow)
    .run();
}

#[derive(Component)]
struct Tank;

#[derive(Component)]
struct Camera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {

    // Light
    commands.spawn((
        PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..default()},
        Transform::from_xyz(4.0, 8.0, 4.0),
        GlobalTransform::default()));

    //Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Plane3d::default().mesh().size(20.0,20.0).subdivisions(10)))),
        MeshMaterial3d(materials.add(StandardMaterial {base_color: GREEN.into(), ..Default::default()})),
        Collider::cuboid(10.0, 0.1, 10.0)
    ));

    //Tank Block
    // let camo_texture = asset_server.load("camo.jpg");

    // let body_entity = commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 3.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(camo_texture.clone()),
    //         perceptual_roughness: 0.8,
    //         metallic: 0.1, 
    //         ..default()
    //     })),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    //     RigidBody::Dynamic,
    //     Collider::cuboid(1.0,0.5,1.5),
    //     ExternalForce::default(),
    //     Tank
    // )).id();

    // let turret_entity = commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 0.5, 1.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(camo_texture.clone()),
    //         perceptual_roughness: 0.8,
    //         metallic: 0.1, 
    //         ..default()
    //     })),
    //     Transform::from_xyz(0.0, 0.75, 0.0),
    //    // RigidBody::Dynamic,
    //     Collider::cuboid(0.5,0.25,0.5),
    //     ExternalForce::default()
    // )).id();

    // let gun_entity = commands.spawn((
    //     Mesh3d(meshes.add(Cylinder::new(0.1, 3.0))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(camo_texture.clone()),
    //         perceptual_roughness: 0.8,
    //         metallic: 0.1, 
    //         ..default()
    //     })),
    //     Transform::from_xyz(0.0, 0.0, -2.0).with_rotation(Quat::from_rotation_x(90.0_f32.to_radians())),
    //     //RigidBody::Dynamic,
    //     Collider::cylinder(1.5,0.1),
    //     ExternalForce::default()
    // )).id();

    // commands.entity(body_entity).add_child(turret_entity);

    // commands.entity(turret_entity).add_child(gun_entity);

    let tank_parent = commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        RigidBody::Dynamic,
        Collider::cuboid(1.1,1.0,1.3),
        Tank
    )).id();

    let tank_model = commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("concept_tank/scene.gltf"))),
        Transform::from_xyz(0.0, -1.0, 0.0).with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians())),
    )).id();

    commands.entity(tank_parent).add_child(tank_model);

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
        Camera3d {
            ..default()
        },
        Transform::from_xyz(-15.0, 15.0, -15.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Camera
    ));
}

fn tank_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Tank>>
)  {
    let mut transform = query.single_mut();
    let mut direction = 0.0;
    let mut rotation = 0.0;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += 1.0;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        direction -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        rotation -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        rotation += 1.0;
    }

    let rotation_speed = 2.0;
    let move_speed = 5.0;

    transform.rotate_y(rotation * rotation_speed * time.delta_secs());
    let forward = transform.forward();
    transform.translation += forward * direction * move_speed * time.delta_secs();
}

fn camera_follow(
    tank_query: Query<&Transform, (With<Tank>, Without<Camera>)>,
    mut cam_query: Query<&mut Transform, (With<Camera>, Without<Tank>)>
)   {
    let tank_transform = tank_query.single();
    let mut cam_transfrom = cam_query.single_mut();

    let behind = tank_transform.translation - tank_transform.forward() * 10.0 + Vec3::Y * 5.0;
    cam_transfrom.translation = behind;
    cam_transfrom.look_at(tank_transform.translation, Vec3::Y);
}