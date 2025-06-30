struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) texture_index: u32, // unused
    // Instance attributes
    @location(3) instance_pos: vec3<f32>,
    @location(4) face: u32,
    @location(5) block_type: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Face orientations (6 directions)
fn face_transform(face: u32, pos: vec3<f32>) -> vec3<f32> {
    if (face == 0u) { // front (z+)
        return vec3<f32>(pos.x, pos.y, 0.5);
    } else if (face == 1u) { // back (z-)
        return vec3<f32>(-pos.x, pos.y, -0.5);
    } else if (face == 2u) { // left (x-)
        return vec3<f32>(-0.5, pos.y, -pos.x);
    } else if (face == 3u) { // right (x+)
        return vec3<f32>(0.5, pos.y, pos.x);
    } else if (face == 4u) { // top (y+)
        return vec3<f32>(pos.x, 0.5, -pos.y);
    } else { // bottom (y-)
        return vec3<f32>(pos.x, -0.5, pos.y);
    }
}

// Atlas UV calculation
fn get_atlas_uvs(block_type: u32, face: u32, base_uv: vec2<f32>) -> vec2<f32> {
    // Atlas layout: 2x2 grid (4 textures)
    let atlas_size = 2.0;
    let tile_size = 1.0 / atlas_size;
    
    // Determine texture index based on block type and face
    var texture_index = 0u;
    var uv = base_uv;
    if (block_type == 0u) { // Grass
        if (face == 4u) { // Top face
            texture_index = 0u; // grass_top
        } else if (face == 5u) { // Bottom face
            texture_index = 2u; // dirt
        } else {
            texture_index = 1u; // grass_side
            uv.y = 1.0 - uv.y; // Flip V for side faces
        }
    } else if (block_type == 1u) { // Dirt
        texture_index = 2u;
    } else if (block_type == 2u) { // Stone
        texture_index = 3u;
    } else {
        texture_index = 0u;
    }
    
    // Calculate atlas position
    let tile_x = f32(texture_index % 2u) * tile_size;
    let tile_y = f32(texture_index / 2u) * tile_size;
    
    // Transform base UV to atlas position
    return vec2<f32>(
        tile_x + uv.x * tile_size,
        tile_y + uv.y * tile_size
    );
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Transform quad to face orientation and world position
    let local = face_transform(model.face, model.position);
    let world = local + model.instance_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world, 1.0);
    // Calculate atlas UVs from block_type and base UVs
    out.tex_coords = get_atlas_uvs(model.block_type, model.face, model.tex_coords);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_atlas, s_atlas, in.tex_coords);
} 