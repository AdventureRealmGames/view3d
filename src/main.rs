use bevy::{
    camera::{visibility::RenderLayers}, core_pipeline::{prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass}, tonemapping::Tonemapping}, light::CascadeShadowConfigBuilder, pbr::ExtendedMaterial, prelude::*
};
use bevy_egui::{
    EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext,
};
use bevy_enhanced_input::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use std::{env, f32::consts::PI};
use view3d::{
    files::{
        CurrentGltfEntity, Directory, EditFileName, FileList, ModelInfo, OpenFile,
        ShowEditFileName, SortMode, check_dir_changed, check_model_loaded, check_open_file_changed,
        home_dir, dir_list_approved_files,
    },
    objects::{EnvironmentMaterial, change_material},
    ui::{UiKeyAction, handle_file_nav_down, handle_file_nav_up, setup_ui, ui_system},
    thumbnails::{ThumbnailCache, ThumbnailQueue, GenerateThumbnail, handle_thumbnail_requests, process_thumbnail_queue, cleanup_thumbnail_cameras},
};


fn main() {
    let args = env::args();
    println!("{:?}", args);
    let dir = if args.len() > 1 {
        Directory(args.last().unwrap_or(".".to_string()))
    } else {
        let home = home_dir();
        Directory(home)
    };

    App::new()
.insert_resource(ClearColor(    Color::srgb_u8(15, 16,17)    ))
        
        .insert_resource(AmbientLight {
            affects_lightmapped_meshes: true,
            color: Color::WHITE,
            brightness: 0.0,
        })
        //.init_resource::<Directory>()
        .insert_resource(dir)
        .init_resource::<ModelInfo>()
        .init_resource::<OpenFile>()
        .init_resource::<CurrentGltfEntity>()
        .init_resource::<EditFileName>()
        .init_resource::<ShowEditFileName>()
        .insert_resource(SortMode::Name)
        .init_resource::<ThumbnailCache>()
        .init_resource::<ThumbnailQueue>()
        .add_message::<GenerateThumbnail>()
        //plugins
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..Default::default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(EnhancedInputPlugin)
        .add_plugins(WireframePlugin::default())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, EnvironmentMaterial>,
        >::default())
        // systems
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, setup_ui)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, check_dir_changed)
        .add_systems(Update, check_open_file_changed)
        .add_systems(Update, handle_thumbnail_requests)
        .add_systems(Update, process_thumbnail_queue)
        .add_systems(Update, cleanup_thumbnail_cameras)
        //observers
        .add_observer(handle_file_nav_up)
        .add_observer(handle_file_nav_down)
        .add_observer(check_model_loaded)
        .add_observer(change_material)
         .add_observer(toggle_wireframe)
        //input
        .add_input_context::<UiKeyAction>()
         .add_input_context::<SystemAction>()
        .run();
}



#[derive(Component)]
pub struct SystemAction;

// Set up the example entities for the 3D scene. The only important thing is a camera which
// renders directly to the window.
fn setup_scene(
    directory: Res<Directory>,
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    _meshes: Res<Assets<Mesh>>,
    _materials: Res<Assets<StandardMaterial>>,
    sort_mode: Res<SortMode>,
    _asset_server: Res<AssetServer>,
    //mut image_assets: &mut Assets<Image>,
    _image_assets: Res<Assets<Image>>,
) {
    let entries = dir_list_approved_files(&directory.0, *sort_mode);

    commands.insert_resource(FileList(entries));

   
  commands.spawn((
        SystemAction,
        actions!( SystemAction[             
             (Action::<ToggleWireframe>::new(), bindings![KeyCode::KeyR]),
            // (
            //     Action::<Pause>::new(),
            //     Press::new(1.0),
            //     bindings![KeyCode::KeyP],
            // )
            ]
        ),
    ));

    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn((
        DirectionalLight {
            //illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            //illuminance: light_consts::lux::DIRECT_SUNLIGHT,
            illuminance: 6_000.,
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
        // Ensure this light only affects the main world, not thumbnails
        RenderLayers::layer(0),
    ));

    // commands.spawn((
    //     PointLight {
    //         intensity: 1_500_000., // lumens
    //         color: Color::WHITE,
    //         shadows_enabled: false,
    //         radius: 0.,
    //         range: 1000.,
    //         ..default()
    //     },
    //     Transform::from_xyz(-10., 10., 10.),
    // ));

    // commands.spawn((
    //     PointLight {
    //         intensity: 1_000_000., // lumens
    //         color: Color::WHITE,
    //         shadows_enabled: false,
    //         radius: 0.,
    //         range: 2000.,
    //         ..default()
    //     },
    //     Transform::from_xyz(-4., -10., -10.),
    // ));

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


    commands
        .spawn((
            // Camera3d::default(),
            Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            PanOrbitCamera::default(),
            Camera3d { ..default() },
            Camera { order: 50, ..default() },
            // Ensure the main camera does not render thumbnail entities on layer 7
            RenderLayers::layer(0),
            
            // EnvironmentMapLight {
            //     intensity: 200.0,
            //     ..EnvironmentMapLight::solid_color(&mut image_assets, Color::WHITE)
            // },
            //Exposure::SUNLIGHT,
            Tonemapping::ReinhardLuminance,
            
        ))
        .insert((DepthPrepass, NormalPrepass, MotionVectorPrepass))
        // .insert(AmbientLight {
        //     affects_lightmapped_meshes: true,
        //     color: Color::WHITE,
        //     brightness: 0.0,
        // })
        ;

    // Egui camera
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 100,
            ..default()
        },
    ));
}

/*
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
    },
};
*/

// pub(super) fn plugin(app: &mut App) {
//     let _ = app;
// }


#[derive(InputAction)]
#[action_output(bool)]
pub struct ToggleWireframe;

pub fn toggle_wireframe(
    _trigger: On<Complete<ToggleWireframe>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = !wireframe_config.global;
}
