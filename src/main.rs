use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    light::CascadeShadowConfigBuilder,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
    window::PrimaryWindow,
};
use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};
use bevy_enhanced_input::{prelude::InputContextAppExt, EnhancedInputPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bytesize::ByteSize;
use std::{f32::consts::PI, fs};
use view3d::{
    files::{
        check_dir_changed, check_open_file_changed, read_directory_files, CurrentGltfEntity, Directory, EditFileName, FileList, OpenFile, ShowEditFileName, SortMode
    },
    style::styled_button,
    ui::{handle_file_nav_down, handle_file_nav_up, setup_ui, ui_system, UiKeyAction},
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            affects_lightmapped_meshes: false,
            color: Color::WHITE,
            brightness: 150.0,
        })
        .init_resource::<Directory>()
        .init_resource::<OpenFile>()
        .init_resource::<CurrentGltfEntity>()
        .init_resource::<EditFileName>()
        .init_resource::<ShowEditFileName>()
        .insert_resource(SortMode::Name)
        //plugins
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..Default::default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(EnhancedInputPlugin)
        // systems
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, setup_ui)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, check_dir_changed)
        .add_systems(Update, check_open_file_changed)
        //observers
        .add_observer(handle_file_nav_up)
        .add_observer(handle_file_nav_down)
        //input
        .add_input_context::<UiKeyAction>()
        .run();
}

// Set up the example entities for the 3D scene. The only important thing is a camera which
// renders directly to the window.
fn setup_scene(
    mut directory: ResMut<Directory>,
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sort_mode: ResMut<SortMode>,
) {
    println!("Dir: {}", directory.0);
    let entries = read_directory_files(&directory.0, *sort_mode);

    commands.insert_resource(FileList(entries));

    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    // Add a light source
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 2., -PI / 4.)),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 7.0,
            maximum_distance: 25.0,
            ..default()
        }
        .build(),
    ));

    /*
        // Cube
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2),
                ..default()
            })),
            Transform::from_xyz(-2.0, 0.5, 0.0),
        ));

        // Sphere
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.8, 0.2),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ));

        // Cylinder
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.8),
                ..default()
            })),
            Transform::from_xyz(2.0, 0.5, 0.0),
        ));

        // Ground plane
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.3),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    */
    // 3D World camera positioned to view the scene

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        PanOrbitCamera::default(),
    ));

    // Egui camera
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}
