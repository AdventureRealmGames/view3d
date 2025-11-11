use bevy::{
    camera::visibility::RenderLayers, color::palettes, core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass}, prelude::*, render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    }
};
use bevy::asset::LoadState;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use crate::objects::ColorOverride;

/// Resource that stores generated thumbnails for file paths
#[derive(Resource, Default)]
pub struct ThumbnailCache {
    pub thumbnails: HashMap<String, Handle<Image>>,
    pub pending: HashMap<String, ThumbnailState>,
}

#[derive(Resource, Default)]
pub struct ThumbnailQueue {
    pub queue: VecDeque<String>,
    pub busy: bool,
}

/// State of a thumbnail being generated
#[derive(Debug, Clone)]
pub enum ThumbnailState {
    Queued,               // Waiting in the queue
    Loading(Entity),      // Entity of the loaded model (unused currently)
    Rendering(Entity),    // Entity of the camera rendering it
    Ready,                // Thumbnail is ready in cache
}

/// Component marking a thumbnail camera
#[derive(Component)]
pub struct ThumbnailCamera {
    pub file_path: String,
    pub frames_to_render: u32,
    pub layer: u8,
}

/// Component marking a model being rendered for thumbnail
#[derive(Component)]
pub struct ThumbnailModel {
    pub file_path: String,
    pub layer: u8,
}

//// Component marking a light used for a thumbnail render
#[derive(Component)]
pub struct ThumbnailLight {
    pub file_path: String,
    pub layer: u8,
}


/// Marker for the thumbnail render layer
pub const THUMBNAIL_LAYER: usize = 7;

/// Size of thumbnail textures
pub const THUMBNAIL_SIZE: u32 = 256;

/// Compute a stable render layer index (1..=31) for a given file path to isolate thumbnail renders.
fn compute_layer(file_path: &str) -> u8 {
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    let hash = hasher.finish();
    // Use layers 1..=31 (reserve 0 for the main world)
    ((hash % 31) as u8) + 1
}

/// Request to generate a thumbnail for a file
#[derive(Message)]
pub struct GenerateThumbnail {
    pub file_path: String,
}

/// System to handle thumbnail generation requests
pub fn handle_thumbnail_requests(
    mut events: MessageReader<GenerateThumbnail>,
    mut cache: ResMut<ThumbnailCache>,
    mut queue: ResMut<ThumbnailQueue>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        let file_path = event.file_path.clone();
        println!("[THUMBNAIL] Received request to generate thumbnail for: {:?}", file_path);

        // Skip if already in cache or pending
        if cache.thumbnails.contains_key(&file_path) {
            println!("[THUMBNAIL] Already in cache, skipping: {:?}", file_path);
            continue;
        }
        if cache.pending.contains_key(&file_path) {
            println!("[THUMBNAIL] Already pending, skipping: {:?}", file_path);
            continue;
        }

        println!("[THUMBNAIL] Queuing thumbnail generation for: {:?}", file_path);

        // Create render target texture now so UI can hold a handle (will be filled when ready)
        let size = Extent3d {
            width: THUMBNAIL_SIZE,
            height: THUMBNAIL_SIZE,
            depth_or_array_layers: 1,
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        image.resize(size);
        let image_handle = images.add(image);
        cache.thumbnails.insert(file_path.clone(), image_handle);
        cache.pending.insert(file_path.clone(), ThumbnailState::Queued);

        // Push into the generation queue
        queue.queue.push_back(file_path.clone());
        println!("[THUMBNAIL] Enqueued: {:?}. Queue length: {}", file_path, queue.queue.len());
    }
}

/// System to drive queued thumbnail generation, processing exactly one at a time
pub fn process_thumbnail_queue(
    mut commands: Commands,
    mut queue: ResMut<ThumbnailQueue>,
    mut cache: ResMut<ThumbnailCache>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    if queue.busy {
        return;
    }

    if let Some(file_path) = queue.queue.pop_front() {
        println!("[THUMBNAIL] Dequeued: {:?}. Starting generation.", file_path);
        queue.busy = true;

        // Use a single shared render layer for all thumbnails
        let layer = THUMBNAIL_LAYER as u8;

        // Prepare scene and rendering resources
        let scene_path = format!("{}#Scene0", file_path);
        let scene = asset_server.load(scene_path);

        // Spawn model targeted for the thumbnail layer
        let model_entity = commands
            .spawn((
                SceneRoot(scene),
                Transform::from_scale(Vec3::splat(1.0)),
                Visibility::Visible,
                RenderLayers::layer(THUMBNAIL_LAYER),
                ThumbnailModel {
                    file_path: file_path.clone(),
                    layer,
                },
                ColorOverride(palettes::tailwind::GRAY_950.into()),
            ))
            .id();
        println!("[THUMBNAIL] Spawned model entity: {:?}", model_entity);

        // Resolve the render target created during enqueue
        let image_handle = cache
            .thumbnails
            .get(&file_path)
            .cloned()
            .expect("Thumbnail image handle must exist");

        // Spawn offscreen camera to render into the image
        let camera_entity = commands
            .spawn((
                Camera3d::default(),
                Camera {
                    order: -10,
                    target: image_handle.clone().into(),
                    clear_color: Color::srgb(0.001, 0.001, 0.001).into(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.420, 1.20).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                RenderLayers::layer(THUMBNAIL_LAYER),
                ThumbnailCamera {
                    file_path: file_path.clone(),
                    frames_to_render: 3,
                    layer,
                },
            ))
            .insert((DepthPrepass, NormalPrepass, MotionVectorPrepass))
            .id();
        println!("[THUMBNAIL] Spawned camera entity: {:?}", camera_entity);

        // Add a light affecting only the thumbnail layer
        commands.spawn((
            DirectionalLight {
                illuminance: 10_000.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                std::f32::consts::PI / 2.0,
                -std::f32::consts::PI / 4.0,
            )),
            RenderLayers::layer(THUMBNAIL_LAYER),
            ThumbnailLight {
                file_path: file_path.clone(),
                layer,
            },
        ));

        // Mark as actively rendering
        cache
            .pending
            .insert(file_path.clone(), ThumbnailState::Rendering(camera_entity));

        println!(
            "[THUMBNAIL] Generation started for: {:?} (remaining in queue: {})",
            file_path,
            queue.queue.len()
        );
    }
}

/// System to clean up thumbnail cameras after rendering
pub fn cleanup_thumbnail_cameras(
    mut commands: Commands,
    mut active_thumbnail_cameras: Query<(Entity, &mut ThumbnailCamera)>,
    mut cache: ResMut<ThumbnailCache>,
    mut queue: ResMut<ThumbnailQueue>,
    models: Query<(Entity, &ThumbnailModel, &SceneRoot)>,
    lights: Query<(Entity, &ThumbnailLight)>,
    children: Query<&Children>,
    mut scene_cameras: Query<&mut Camera>,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut camera) in active_thumbnail_cameras.iter_mut() {
        // Wait until the glTF scene for this thumbnail is fully loaded
        let mut scene_loaded = false;
        for (model_entity, model, scene_root) in models.iter() {
            if model.file_path == camera.file_path {
                let state = asset_server.get_load_state(scene_root.0.id());
                if let Some(LoadState::Loaded) = state {
                    // Ensure the entire spawned scene hierarchy is on the thumbnail render layer,
                    // otherwise the camera won't see child meshes.
                    apply_layers_recursive(model_entity, THUMBNAIL_LAYER, &mut commands, &children);

                    // Disable any cameras that may have been loaded from the GLTF scene to prevent
                    // them from rendering to the main window.
                    disable_cameras_recursive(model_entity, &mut scene_cameras, &children);

                    scene_loaded = true;
                    break;
                }
            }
        }

        if !scene_loaded {
            // Defer rendering countdown until the assets are loaded to avoid capturing just the clear color.
            continue;
        }

        if camera.frames_to_render > 0 {
            camera.frames_to_render -= 1;
            println!(
                "[THUMBNAIL] Camera {:?} rendering frame {} for: {:?}",
                entity,
                3 - camera.frames_to_render,
                camera.file_path
            );
        } else {
            println!("[THUMBNAIL] Cleaning up camera for: {:?}", camera.file_path);

            // Mark as ready in cache
            if let Some(state) = cache.pending.get_mut(&camera.file_path) {
                *state = ThumbnailState::Ready;
                println!("[THUMBNAIL] Marked as ready: {:?}", camera.file_path);
            }

            // Despawn camera
            commands.entity(entity).despawn();
            println!("[THUMBNAIL] Despawned camera entity: {:?}", entity);

            // Despawn associated models for this file only
            for (model_entity, model, _scene_root) in models.iter() {
                if model.file_path == camera.file_path {
                    commands.entity(model_entity).despawn();
                    println!("[THUMBNAIL] Despawned model entity: {:?}", model_entity);
                }
            }

            // Despawn associated lights for this file only
            for (light_entity, light) in lights.iter() {
                if light.file_path == camera.file_path {
                    commands.entity(light_entity).despawn();
                    println!("[THUMBNAIL] Despawned light entity: {:?}", light_entity);
                }
            }

            // Allow the next item in the queue to proceed
            queue.busy = false;
        }
    }
}

//// Recursively apply the thumbnail render layer to an entity and all its descendants.
fn apply_layers_recursive(entity: Entity, layer: usize, commands: &mut Commands, children_query: &Query<&Children>) {
    commands.entity(entity).insert(RenderLayers::layer(layer));
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            apply_layers_recursive(child, layer, commands, children_query);
        }
    }
}

//// Recursively disable any Camera components under an entity hierarchy (e.g., GLTF cameras)
fn disable_cameras_recursive(entity: Entity, scene_cameras: &mut Query<&mut Camera>, children_query: &Query<&Children>) {
    if let Ok(mut cam) = scene_cameras.get_mut(entity) {
        cam.is_active = false;
    }
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            disable_cameras_recursive(child, scene_cameras, children_query);
        }
    }
}

/// Get or request a thumbnail for a file path
pub fn get_thumbnail(
    file_path: &str,
    cache: &ThumbnailCache,
    events: &mut MessageWriter<GenerateThumbnail>,
) -> Option<Handle<Image>> {
    if let Some(handle) = cache.thumbnails.get(file_path) {
        println!("[THUMBNAIL] Found cached thumbnail for: {:?}", file_path);
        Some(handle.clone())
    } else {
        println!("[THUMBNAIL] Requesting thumbnail generation for: {:?}", file_path);
        // Request generation
        events.write(GenerateThumbnail {
            file_path: file_path.to_string(),
        });
        None
    }
}
