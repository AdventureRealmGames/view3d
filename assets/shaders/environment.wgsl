#import "shaders/custom_material_import.wgsl"::COLOR_MULTIPLIER
#import bevy_render::view::View

#import bevy_pbr::{
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::globals,
    prepass_utils,
    mesh_view_bindings::depth_prepass_texture,
    forward_io::{VertexOutput,FragmentOutput},
    mesh_view_bindings::view,
    mesh_view_bindings::lights,
    utils::PI,
}

// Simple noise functions for foam variation
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    
    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value;
}

//#import bevy_shader_utils::mock_fresnel;


@group(#{MATERIAL_BIND_GROUP}) @binding(200) var<uniform> material_color: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(201) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(202) var material_color_sampler: sampler;

// struct WaveData {
//     wave_heights: vec4<f32>,
//     wave_lengths: vec4<f32>,
//     wave_speeds: vec4<f32>,
//     wave_angles: vec4<f32>,
// }

//@group(#{MATERIAL_BIND_GROUP}) @binding(203) var<uniform> wave_data: WaveData;
@group(#{MATERIAL_BIND_GROUP}) @binding(204) var<uniform> time: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(205) var<uniform> texture_mix_amount: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(206) var<uniform> ambient_light: f32;

// Constants
const PI: f32 = 3.14159265359;

// Gerstner wave calculation functions
fn evaluate_gerstner_wave(position: vec3<f32>, wave_height: f32, wave_length: f32, wave_speed: f32, wave_angle: f32, time: f32) -> f32 {
    let direction = vec3<f32>(cos(radians(wave_angle)), 0.0, sin(radians(wave_angle)));
    let wavenumber = 2.0 * PI / wave_length;
    let phase = wavenumber * dot(position, direction) - wave_speed * time;
    return wave_height * sin(phase);
}

// Ocean parameters
const OCEAN_BASE_ALPHA: f32 = 1.0;
const FADE_COLOR: vec3<f32> = vec3<f32>(0.001, 0.121, 0.316); // Deep ocean blue
//const SURFACE_COLOR: vec3<f32> = vec3<f32>(0.4, 0.8, 1.0); // Surface blue
const DEPTH_FADE_STRENGTH: f32 = 2.0; // Controls how quickly objects fade with depth
const MAX_DEPTH_FADE: f32 = 28.0; // Maximum depth distance for fading

@fragment
fn fragment(
    @builtin(sample_index) sample_index: u32,
   // @builtin(front_facing) is_front: bool,      
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {

    // get pbr lighting
    var pbr_input = pbr_input_from_standard_material(mesh, true);
    var out: FragmentOutput;
    let base_material = pbr_input.material.base_color;
    let lit = apply_pbr_lighting(pbr_input);
    out.color = (ambient_light * base_material) + (lit * (1.0 - ambient_light));
    // we can optionally modify the lit color before post-processing is applied
    //out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);



    // // Calculate depth effects
    let z_depth = bevy_pbr::prepass_utils::prepass_depth(mesh.position, sample_index);
    
    
    let pre_scale = 500.0;    
    let contrast_power = 1.1;
    let shift = 0.5;
    let z_depth_prescaled = smoothstep(0.0,1.0,z_depth * pre_scale);
    let z_depth_scaled = (( z_depth_prescaled - shift) * contrast_power) + shift;
    let depth_clamped = smoothstep(0.0,1.0,z_depth_scaled);
    //let depth_clamped = clamp(z_depth_scaled,0.0,1.0);
    //let depth_calc = pow(depth_scaled, 1.0 / contrast_power);
    let z_depth_factor = depth_clamped;
    
    //let camera_distance = length(view.world_position.xyz - mesh.world_position.xyz);
    
    //  let depth_scale = 10.0;
    //  let shift = 0.00002618;
    //  let raw_depth = ((depth - shift ) * depth_scale) ;
    //  let contrast_power = 6.0;
    //  let depth_calc = pow(raw_depth, 1.0 / contrast_power);
    //  let depth_factor = depth_calc + shift;

    // Calculate dynamic ocean height at this position
    let ocean_height = 0.0;// calculate_ocean_height_at_position(mesh.world_position.xyz, time);
    
    // Calculate world depth below dynamic ocean surface
    let world_y = mesh.world_position.y;
    let depth_below_surface = max(0.0, ocean_height - world_y);
    let normalized_depth = clamp(depth_below_surface / MAX_DEPTH_FADE, 0.0, 1.0);
    let depth_with_strength = pow(normalized_depth, 1.0 / DEPTH_FADE_STRENGTH);
    let depth_mix_clamped = smoothstep(0.0,1.0,depth_with_strength);
   
    let depth_fix_alpha = (depth_mix_clamped);// * material_color.rgb; 
    //let depth_mix = mix(material_color.rgb, vec3<f32>(1.0,1.0,1.0), depth_fix);
    //let final_mix = depth_mix * out.color.rgb; 
    let pbr_color = out.color.rgb;//vec3<f32>(0.94, 0.99, 0.99,); //material_color.rgb;//out.color.rgb;
    let ocean_fade_color = vec3<f32>(0.004, 0.009, 0.09,);// material_color.rgb;
    //let depth_layer = vec4<f32>(pbr_color,depth_fix_alpha);
    let alpha = depth_fix_alpha;//depth_factor; //depth_fix_alpha    
    let depth_mix = (ocean_fade_color * alpha) + (pbr_color * (1.0-alpha));
    //let depth_mix = pbr_color;
    
    // if world_y < ocean_height {
    //     if view.world_position.y < ocean_height {
    //         //soomehow desaturated would be cool
    //         let depth_alpha = 1.0 - z_depth_factor;
    //         let pre_mix = (ocean_fade_color * depth_alpha) + (depth_mix * (1.0-depth_alpha));
    //         let underwater_scaled = (z_depth_factor * 0.8) - 0.4;
    //         let scale_depth_alpha = clamp(underwater_scaled, 0.0, 1.0);
    //         let final_mix = (pbr_color * scale_depth_alpha) + (ocean_fade_color * (1.0-scale_depth_alpha));        
    //         //return vec4<f32>(scale_depth_alpha,scale_depth_alpha,scale_depth_alpha, 1.0);
    //         return vec4<f32>(final_mix, 1.0);
    //     } else {
    //         let depth_alpha = 1.0 - z_depth_factor; //- z_depth_factor;
    //         let final_mix = (ocean_fade_color * depth_alpha) + (depth_mix * (1.0-depth_alpha));        
    //         return vec4<f32>(final_mix, 1.0);
    //     }        
    // } else {
        return vec4<f32>(depth_mix, 1.0);
       
    //}
    
    //return vec4<f32>(out.color.rgb, 1.0);
    
}
