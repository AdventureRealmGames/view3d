use bevy::{
    ecs::relationship::RelationshipSourceCollection,
    light::CascadeShadowConfigBuilder,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
    window::PrimaryWindow,
};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::fs::ReadDir;
use std::fs::{self};

use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

#[derive(Resource)]
pub struct Directory(pub String);

impl Default for Directory {
    fn default() -> Self {
        Self(".".to_string())
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct OpenFile(pub String);

impl Default for OpenFile {
    fn default() -> Self {
        Self("".to_string())
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Name,
    Size,
    Date,
}

#[derive(Resource)]
pub struct FileList(pub Vec<FileEntry>);

#[derive(Resource)]
pub struct EditFileName(pub String);
impl Default for EditFileName {
    fn default() -> Self {
        Self("".to_string())
    }
}

#[derive(Resource)]
pub struct ShowEditFileName(pub bool);
impl Default for ShowEditFileName {
    fn default() -> Self {
        Self(false)
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub last_modified: u64,
    //pub size: u64
}

pub fn file_dir_path(dir: String, file: String) -> String {
    let path = std::path::Path::new(&dir).join(file);
    path.to_str().unwrap_or("").to_string()
}

pub fn check_dir_changed(
    dir: Res<Directory>,
    mut file_list: ResMut<FileList>,
    sort_mode: Res<SortMode>,
) {
    if dir.is_changed() || sort_mode.is_changed() {
        file_list.0 = read_directory_files(&dir.0, *sort_mode);
    }
}

// pub fn move_file(src: String, dest: String) -> Result {
//     match fs::rename(src, dest) {
//         Ok(_) => Ok(()),
//         Err(e) => bevy::ecs::error::BevyError(e.to_string())
//     }
// }

pub fn read_directory_files(path: &str, sort_mode: SortMode) -> Vec<FileEntry> {
    // Define accepted file extensions
    let accepted_extensions = ["glb", "gltf"];

    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut items: Vec<(bool, FileEntry)> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    if e.file_name().to_string_lossy().starts_with(".") {
                        return false;
                    }
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
                    let fe = FileEntry {
                        name: e.file_name().to_string_lossy().to_string(),
                        last_modified: e
                            .metadata()
                            .unwrap()
                            .modified()
                            .unwrap()
                            .elapsed()
                            .unwrap_or_default()
                            .as_secs(),
                    };
                    (is_dir, fe)
                })
                .collect();

            match sort_mode {
                SortMode::Name => items.sort_by(|a, b| {
                    match (a.0, b.0) {
                        (true, false) => std::cmp::Ordering::Less, // dirs before files
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()),
                    }
                }),
                SortMode::Size => todo!(),
                SortMode::Date => items.sort_by(|a, b| {
                    match (a.0, b.0) {
                        (true, false) => std::cmp::Ordering::Less, // dirs before files
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.1.last_modified.cmp(&b.1.last_modified),
                    }
                }),
            }

            items
                .into_iter()
                .map(|(_, file_entry)| file_entry)
                .collect()
        }
        Err(e) => {
            error!("Failed to read directory '{}': {}", path, e);
            Vec::new()
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentGltfEntity(pub Option<Entity>);

pub fn check_open_file_changed(
    mut commands: Commands,
    open_file: Res<OpenFile>,
    asset_server: Res<AssetServer>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    mut current_gltf: ResMut<CurrentGltfEntity>,
) {
    if open_file.is_changed() {
        // Despawn the old GLTF entity if it exists
        if let Some(old_entity) = current_gltf.0 {
            println!("Despawning old GLTF entity: {:?}", old_entity);
            commands.entity(old_entity).despawn();
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
            ))
            .id();

        // Store the new entity ID
        current_gltf.0 = Some(land_entity);
    }
}



pub fn home_dir() -> String {
    //let path = "";

    let user_dirs = UserDirs::new();
    // let desktop_dir = match &user_dirs {
    //     Some(user_dirs) => user_dirs.desktop_dir(),
    //     None => {
    //         println!("Could not retrieve user directories.");
    //         None
    //     }
    // }
    // .unwrap();

    let home_dir = match &user_dirs {
        Some(user_dirs) => user_dirs.home_dir(),
        None => {
            println!("Could not retrieve user directories.");
            panic!()
        }
    };

    //let p = home_dir.join(path);
    home_dir.to_string_lossy().to_string()
    
}