use std::f32::consts::PI;

use bevy::{
    ecs::relationship::RelationshipSourceCollection,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::camera::Viewport,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
    window::PrimaryWindow,
};

use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_render::view::RenderLayers;
use view3d::list_dir;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.25, 0.25, 0.25)))
        .insert_resource(AmbientLight {
            affects_lightmapped_meshes: false,
            color: Color::WHITE,
            brightness: 150.0,
        })
        .init_resource::<Directory>()
        .init_resource::<OpenFile>()
        .init_resource::<CurrentGltfEntity>()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..Default::default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_scene)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, check_dir_changed)
        .add_systems(Update, check_open_file_changed)
        .run();
}

#[derive(Resource)]
struct Directory(String);
impl Default for Directory {
    fn default() -> Self {
        Self(".".to_string())
    }
}

#[derive(Resource)]
struct OpenFile(String);
impl Default for OpenFile {
    fn default() -> Self {
        Self("".to_string())
    }
}

#[derive(Resource, Default)]
struct CurrentGltfEntity(Option<Entity>);

/// Helper function to read directory contents with proper error handling
fn read_directory_files(path: &str) -> Vec<String> {
    // Define accepted file extensions
    let accepted_extensions = ["glb", "gltf"];

    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<(bool, String)> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    // Allow directories
                    //TODO hide hidden folders
                    if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        return true;
                    }
                    // Check if file has an accepted extension
                    e.path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| accepted_extensions.contains(&ext))
                        .unwrap_or(false)
                })
                // .map(|e| e.file_name().to_string_lossy().to_string())
                //.collect();
                .map(|e| {
                    let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                    (is_dir, e.file_name().to_string_lossy().to_string())
                })
                .collect();

            items.sort_by(|a, b| {
                match (a.0, b.0) {
                    (true, false) => std::cmp::Ordering::Less, // dirs before files
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
                }
            });

            items.into_iter().map(|(_, name)| name).collect()
        }
        Err(e) => {
            error!("Failed to read directory '{}': {}", path, e);
            Vec::new()
        }
    }
}

fn check_dir_changed(dir: Res<Directory>, mut file_list: ResMut<FileList>) {
    if dir.is_changed() {
        file_list.0 = read_directory_files(&dir.0);
    }
}

fn check_open_file_changed(
    mut commands: Commands,
    open_file: Res<OpenFile>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_gltf: ResMut<CurrentGltfEntity>,
) {
    if open_file.is_changed() {
        // Despawn the old GLTF entity if it exists
        if let Some(old_entity) = current_gltf.0 {
            println!("Despawning old GLTF entity: {:?}", old_entity);
            commands.entity(old_entity).despawn_recursive();
        }

        let file_name = format!("{}#Scene0", open_file.0);
        println!("Filename: {}", file_name);
        let scene = asset_server.load(file_name);
        let scale = 1.0;
        let land_entity = commands
            .spawn((
                SceneRoot(scene.clone()), //#Scene0
                Visibility::Visible,
                //transform: Transform::from_scale(Vec3::new(0.1,0.1,0.1)),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    rotation: Default::default(),
                    scale: Vec3::new(scale, scale, scale),
                },
                //RigidBody::Static,
                // we are now adding this per object in blender.
                //ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
                // Mass(1.0),
                //COLIDER
                //Collider::trimesh_from_mesh(mesh)
                //AsyncSceneCollider { shape: Some(ComputedColliderShape::TriMesh(Default::default())), named_shapes: Default::default() },
                //RigidBody::Fixed {},
                //collider
            ))
            .id();

        // Store the new entity ID
        current_gltf.0 = Some(land_entity);
    }
}

fn styled_button(ui: &mut egui::Ui, text: &str, dir: bool, is_selected: bool) -> egui::Response {
    // Define colors for different states
    let (bg_color, hover_color, text_color) = match (dir, is_selected) {
        // Selected file - blue theme
        (false, true) => (
            egui::Color32::from_rgb(80, 80, 90),
            egui::Color32::from_rgb(70, 130, 21),
            egui::Color32::WHITE,
        ),
        // Regular file -
        (false, false) => (
            egui::Color32::from_rgb(28, 29, 30),
            egui::Color32::from_rgb(20, 20, 20),
            egui::Color32::from_rgb(200, 202, 203),
        ),
        // Directory - yellow/gold theme
        (true, _) => (
            egui::Color32::from_rgb(50, 52, 60),
            egui::Color32::from_rgb(25, 20, 10),
            egui::Color32::from_rgb(200, 202, 203),
        ),
    };

    // Create custom button style
    let button = egui::Button::new(egui::RichText::new(format!("{} ", text)).color(text_color))
        .fill(bg_color)
        .stroke(egui::Stroke::new(
            1.0,
            if is_selected {
                egui::Color32::from_rgb(40, 80, 150)
            } else {
                egui::Color32::from_gray(180)
            },
        ))
        .rounding(4.0)
        .min_size(egui::vec2(200.0, 30.0));

    // Apply hover color
    let response = ui.add(button);

    // Custom hover effect using style
    if response.hovered() {
        // ui.painter().rect_filled(response.rect, 4.0, hover_color);
    }

    response
}

#[derive(Default)]
pub struct MyState {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
}

type DialogResponse = Option<rfd::FileHandle>;

// This function runs every frame. Therefore, updating the viewport after drawing the gui.
// With a resource which stores the dimensions of the panels, the update of the Viewport can
// be done in another system.
fn ui_system(
    mut directory: ResMut<Directory>,
    mut open_file: ResMut<OpenFile>,
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    mut state: Local<MyState>,
    mut file_dialog: Local<Option<Task<DialogResponse>>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut file_list: ResMut<FileList>,
) -> Result {
    // Poll the file dialog task FIRST, before any early returns
    if let Some(file_response) = file_dialog
        .as_mut()
        .and_then(|task| block_on(poll_once(task)))
    {
        state.picked_path = file_response.map(|path| path.path().display().to_string());
        *file_dialog = None;
    }

    let ctx = contexts.ctx_mut()?;

    let mut left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Left resizeable panel");

            // Add text input section
            ui.horizontal(|ui| {
                ui.label("Directory:");
                ui.text_edit_singleline(&mut directory.0);
            });
            ui.label(format!("Open File {}", open_file.0));
            ui.separator();
            ui.label(format!("Browsing {}", directory.0));

            ui.label("Drag-and-drop files onto the window!");

            if ui.button("Open file…").clicked() {
                *file_dialog = Some(
                    AsyncComputeTaskPool::get().spawn(rfd::AsyncFileDialog::new().pick_file()),
                );
            }

            ui.separator();

            if ui.button("Up").clicked() {
                let path = std::fs::canonicalize(&directory.0)
                    .unwrap_or_else(|_| std::path::PathBuf::from(&directory.0));
                if let Some(parent) = path.parent() {
                    directory.0 = parent.to_string_lossy().to_string();
                } else {
                    warn!("Cannot navigate up from directory: {}", directory.0);
                }
            }

            if let Some(picked_path) = &state.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            // Show dropped files (if any):
            if !state.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &state.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }

            if ui.button("Refresh").clicked() {
                file_list.0 = read_directory_files(&directory.0);
            }

            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for filename in &file_list.0 {
                    //    ui.label(entry);
                    // if ui.button(format!("{}", filename)).clicked() {
                    //     let path = std::path::Path::new(&directory.0).join(filename);
                    //     if path.is_dir() {
                    //         directory.0 = path.to_str().unwrap_or(&directory.0).to_string();
                    //     } else {
                    //         open_file.0 = path.to_str().unwrap_or("").to_string();
                    //     }
                    //     // let md = std::fs::metadata(filename)
                    //     // if std::fs::
                    //     // // Handle the button click
                    //     println!("You clicked: {} ", filename,);
                    //     // For example, you could trigger opening, previewing, etc.
                    // }
                    let path = std::path::Path::new(&directory.0).join(filename);
                    let is_selected = open_file.0 == path.to_str().unwrap_or("").to_string();

                    let response = styled_button(
                        ui,
                        format!("{}", filename).as_ref(),
                        path.is_dir(),
                        is_selected,
                    );

                    // Handle click
                    if response.clicked() {
                        if path.is_dir() {
                            directory.0 = path.to_str().unwrap_or(&directory.0).to_string();
                        } else {
                            open_file.0 = path.to_str().unwrap_or("").to_string();
                        }
                    }
                }
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width(); // height is ignored, as the panel has a hight of 100% of the screen

    // Collect dropped files:
    // ctx.input(|i| {
    //     if !i.raw.dropped_files.is_empty() {
    //         state.dropped_files.clone_from(&i.raw.dropped_files);
    //     }
    // });

    // ctx.input(|i| {
    //     if i.raw.modifiers.ctrl {
    //         info!("ctrl pressed");
    //     }
    // });

    let mut right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Right resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width(); // height is ignored, as the panel has a height of 100% of the screen

    let mut top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Top resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height(); // width is ignored, as the panel has a width of 100% of the screen
    let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Bottom resizeable panel");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height(); // width is ignored, as the panel has a width of 100% of the screen

    // Scale from logical units to physical units.
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();
    bottom *= window.scale_factor();

    // -------------------------------------------------
    // |  left   |            top   ^^^^^^   |  right  |
    // |  panel  |           panel  height   |  panel  |
    // |         |                  vvvvvv   |         |
    // |         |---------------------------|         |
    // |         |                           |         |
    // |<-width->|          viewport         |<-width->|
    // |         |                           |         |
    // |         |---------------------------|         |
    // |         |          bottom   ^^^^^^  |         |
    // |         |          panel    height  |         |
    // |         |                   vvvvvv  |         |
    // -------------------------------------------------
    //
    // The upper left point of the viewport is the width of the left panel and the height of the
    // top panel
    //
    // The width of the viewport the width of the top/bottom panel
    // Alternative the width can be calculated as follow:
    // size.x = window width - left panel width - right panel width
    //
    // The height of the viewport is:
    // size.y = window height - top panel height - bottom panel height
    //
    // Therefore we use the alternative for the width, as we can callculate the Viewport as
    // following:

    let pos = UVec2::new(left as u32, top as u32);
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(right as u32, bottom as u32);

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    Ok(())
}

#[derive(Resource)]
struct FileList(Vec<String>);

// Set up the example entities for the 3D scene. The only important thing is a camera which
// renders directly to the window.
fn setup_scene(
    mut directory: ResMut<Directory>,
    mut commands: Commands,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Dir: {}", directory.0);
    let entries = read_directory_files(&directory.0);

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
