use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::color::palettes::css::{RED, WHITE};
use bevy::math::ops::sqrt;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::pbr::wireframe::Wireframe;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
    MaterialPipeline, MaterialPipelineKey, OpaqueRendererMethod,
};
//use bevy::render::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, FrontFace,
    RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::scene::SceneInstanceReady;
use bevy::shader::ShaderRef;
//use bevy::render::view::RenderLayers;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::{light::NotShadowCaster, prelude::*};
use bevy::{prelude::*, reflect::TypePath};
use bevy_render::render_resource::ShaderType;
use rand::prelude::*;

use std::f32::consts::PI;

/// This is added to a [`SceneRoot`] and will cause the [`StandardMaterial::base_color`]
/// of all materials to be overwritten
#[derive(Component)]
pub struct ColorOverride(pub Color);

pub fn change_material(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    color_override: Query<&ColorOverride>,
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    mut ext_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, EnvironmentMaterial>>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    // Get the `ColorOverride` of the entity, if it does not have a color override, skip
    let Ok(color_override) = color_override.get(trigger.target()) else {
        return;
    };

    // Iterate over all children recursively
    for descendants in children.iter_descendants(trigger.target()) {
        // Get the material of the descendant
        if let Some(material) = mesh_materials
            .get(descendants)
            .ok()
            .and_then(|id| asset_materials.get_mut(id.id()))
        {
            // Create a copy of the material and override base color
            // If you intend on creating multiple models with the same tint, it
            // is best to cache the handle somewhere, as having multiple materials
            // that are identical is expensive
            let mut new_material = material.clone();
            new_material.base_color = color_override.0;

            // Override `MeshMaterial3d` with new material
            commands
                .entity(descendants)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                //.insert(MeshMaterial3d(asset_materials.add(new_material)));
                .insert(MeshMaterial3d(ext_materials.add(ExtendedMaterial {
                    base: material.clone(),
                    extension: EnvironmentMaterial {
                        //color: LinearRgba::BLUE,
                        color: LinearRgba::new(0.01, 0.01, 0.12, 1.00),
                        //color_texture: Some(asset_server.load("environment/ocean-06.png")),
                        color_texture: None,
                        ambient_light: 0.42,
                        // color_texture: Some(asset_server.load("branding/icon.png")),
                       
                        time: time.elapsed_secs_wrapped(),
                        texture_mix_amount: 0.0,
                        alpha_mode: AlphaMode::Blend,
                    },
                })));
        }
    }
}

const ENVIRONMENT_SHADER_ASSET_PATH: &str = "shaders/environment.wgsl";

// This struct defines the data that will be passed to your shader
//#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct EnvironmentMaterial {
    #[uniform(200)]
    pub color: LinearRgba,
    #[texture(201)]
    #[sampler(202)]
    pub color_texture: Option<Handle<Image>>,
    #[uniform(204)]
    pub time: f32,
    #[uniform(205)]
    pub texture_mix_amount: f32,
    #[uniform(206)]
    pub ambient_light: f32,
    //#[uniform(207)]
    pub alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl MaterialExtension for EnvironmentMaterial {
    fn fragment_shader() -> ShaderRef {
        ENVIRONMENT_SHADER_ASSET_PATH.into()
    }

    // fn alpha_mode() -> Option<AlphaMode> {
    //    // self.alpha_mode
    //    Some(AlphaMode::Opaque)
    // }

    /* fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;

        // Force alpha blending for all instances of this material
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut() {
                if let Some(color_target_state) = target {
                    color_target_state.blend = Some(BlendState::ALPHA_BLENDING);
                }
            }
        }

        Ok(())
    } */
}

fn update_environment_material_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, EnvironmentMaterial>>>,
) {
    let current_time = time.elapsed_secs_wrapped();
    for (_, material) in materials.iter_mut() {
        material.extension.time = current_time;
    }
}

