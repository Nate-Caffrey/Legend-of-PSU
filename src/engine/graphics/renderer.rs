use std::borrow::Cow;
use wgpu;
use wgpu::util::DeviceExt;
use crate::engine::graphics::{vertex::Vertex, texture::Texture};
use crate::game::world::camera::Camera;
use glam::{Vec3, Mat4, Vec4};
use crate::engine::graphics::vertex::BlockFaceInstance;

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub depth_texture: wgpu::Texture,
    // Occlusion culling support
    pub depth_pyramid: wgpu::Texture,
    pub depth_pyramid_mip_levels: u32,
}

impl Renderer {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        size: winit::dpi::PhysicalSize<u32>,
        texture: &crate::engine::graphics::texture::Texture,
    ) -> Self {
        let surface_caps = surface.get_capabilities(adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        // Don't configure surface here - it's already configured in the app

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/shader.wgsl"))),
        });

        // Camera setup
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(64),
                },
                count: None,
            }],
        });

        let camera = Camera::new();
        let camera_view_proj = camera.create_view_proj(size.width as f32 / size.height as f32);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_view_proj]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout with camera and texture
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), BlockFaceInstance::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Texture"),
            view_formats: &[],
        });

        // Occlusion culling support
        let depth_pyramid_mip_levels = (size.width.max(size.height) as f32).log2().ceil() as u32;
        let depth_pyramid = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: depth_pyramid_mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("Depth Pyramid"),
            view_formats: &[],
        });

        Self {
            device,
            queue,
            config,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            depth_texture,
            depth_pyramid,
            depth_pyramid_mip_levels,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, surface: &wgpu::Surface) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            surface.configure(&self.device, &self.config);
            
            // Recreate depth texture
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Depth Texture"),
                view_formats: &[],
            });

            // Recreate depth pyramid
            let new_mip_levels = (new_size.width.max(new_size.height) as f32).log2().ceil() as u32;
            self.depth_pyramid = self.device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: new_mip_levels,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                label: Some("Depth Pyramid"),
                view_formats: &[],
            });
            self.depth_pyramid_mip_levels = new_mip_levels;
        }
    }

    fn aabb_in_frustum(aabb_min: Vec3, aabb_max: Vec3, frustum_planes: &[Vec4; 6]) -> bool {
        // Test AABB against all 6 frustum planes
        for plane in frustum_planes.iter() {
            // For each plane, compute the positive vertex (furthest in normal direction)
            let p = Vec3::new(
                if plane.x >= 0.0 { aabb_max.x } else { aabb_min.x },
                if plane.y >= 0.0 { aabb_max.y } else { aabb_min.y },
                if plane.z >= 0.0 { aabb_max.z } else { aabb_min.z },
            );
            // If the positive vertex is outside the plane, the box is outside
            if plane.x * p.x + plane.y * p.y + plane.z * p.z + plane.w < 0.0 {
                return false;
            }
        }
        true
    }

    fn extract_frustum_planes(mat: &Mat4) -> [Vec4; 6] {
        // Extract frustum planes from a projection-view matrix (in world space)
        let m = mat.to_cols_array_2d();
        [
            // Left
            Vec4::new(m[0][3] + m[0][0], m[1][3] + m[1][0], m[2][3] + m[2][0], m[3][3] + m[3][0]),
            // Right
            Vec4::new(m[0][3] - m[0][0], m[1][3] - m[1][0], m[2][3] - m[2][0], m[3][3] - m[3][0]),
            // Bottom
            Vec4::new(m[0][3] + m[0][1], m[1][3] + m[1][1], m[2][3] + m[2][1], m[3][3] + m[3][1]),
            // Top
            Vec4::new(m[0][3] - m[0][1], m[1][3] - m[1][1], m[2][3] - m[2][1], m[3][3] - m[3][1]),
            // Near
            Vec4::new(m[0][3] + m[0][2], m[1][3] + m[1][2], m[2][3] + m[2][2], m[3][3] + m[3][2]),
            // Far
            Vec4::new(m[0][3] - m[0][2], m[1][3] - m[1][2], m[2][3] - m[2][2], m[3][3] - m[3][2]),
        ]
    }

    fn calculate_chunk_distance(chunk_pos: Vec3, camera_pos: Vec3) -> f32 {
        (chunk_pos - camera_pos).length_squared()
    }

    fn is_chunk_occluded(chunk_pos: Vec3, chunk_size: f32, camera_pos: Vec3, camera_forward: Vec3) -> bool {
        // Simple occlusion test: check if chunk is behind camera or too far
        let chunk_center = chunk_pos + Vec3::splat(chunk_size * 0.5);
        let to_chunk = chunk_center - camera_pos;
        
        // If chunk is behind camera, it's occluded
        if to_chunk.dot(camera_forward) < -chunk_size {
            return true;
        }
        
        // If chunk is too far, consider it occluded (distance-based culling)
        let distance = to_chunk.length();
        if distance > 100.0 { // Adjust this value based on your view distance
            return true;
        }
        
        false
    }

    fn is_chunk_fully_surrounded(chunk: &crate::game::world::chunk::Chunk, chunk_manager: &crate::game::world::chunk_manager::ChunkManager) -> bool {
        let pos = chunk.position;
        let cs = crate::game::world::chunk::CHUNK_SIZE as f32;
        let neighbor_offsets = [
            (cs, 0.0, 0.0),   // +X
            (-cs, 0.0, 0.0),  // -X
            (0.0, cs, 0.0),   // +Y
            (0.0, -cs, 0.0),  // -Y
            (0.0, 0.0, cs),   // +Z
            (0.0, 0.0, -cs),  // -Z
        ];
        for (dx, dy, dz) in neighbor_offsets.iter() {
            let neighbor_pos = (pos.x + dx, pos.y + dy, pos.z + dz);
            let chunk_key = (
                (neighbor_pos.0 / cs) as i32,
                (neighbor_pos.1 / cs) as i32,
                (neighbor_pos.2 / cs) as i32,
            );
            if let Some(neighbor) = chunk_manager.loaded.get(&chunk_key) {
                // Check if the touching face is fully solid
                if !Renderer::is_face_fully_solid(chunk, neighbor, *dx, *dy, *dz) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn is_face_fully_solid(chunk: &crate::game::world::chunk::Chunk, neighbor: &crate::game::world::chunk::Chunk, dx: f32, dy: f32, dz: f32) -> bool {
        let cs = crate::game::world::chunk::CHUNK_SIZE;
        // For each block on the face, check if the neighbor's touching block is solid
        for x in 0..cs {
            for y in 0..cs {
                for z in 0..cs {
                    let (cx, cy, cz) = (x, y, z);
                    let (nx, ny, nz) = match (dx, dy, dz) {
                        (d, 0.0, 0.0) if d > 0.0 => (0, y, z), // +X face
                        (d, 0.0, 0.0) if d < 0.0 => (cs - 1, y, z), // -X face
                        (0.0, d, 0.0) if d > 0.0 => (x, 0, z), // +Y face
                        (0.0, d, 0.0) if d < 0.0 => (x, cs - 1, z), // -Y face
                        (0.0, 0.0, d) if d > 0.0 => (x, y, 0), // +Z face
                        (0.0, 0.0, d) if d < 0.0 => (x, y, cs - 1), // -Z face
                        _ => continue,
                    };
                    if !neighbor.blocks[nx][ny][nz].is_solid() {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn render(
        &self,
        surface: &wgpu::Surface,
        camera: &Camera,
        texture: &Texture,
        chunks: &[&crate::game::world::chunk::Chunk],
        chunk_manager: &crate::game::world::chunk_manager::ChunkManager,
    ) -> Result<(), wgpu::SurfaceError> {
        let frame = surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Update camera buffer
        let aspect = self.config.width as f32 / self.config.height as f32;
        let view_proj = camera.create_view_proj(aspect);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[view_proj]));
        let view_proj_mat = camera.view_proj_mat(aspect);
        let frustum_planes = Renderer::extract_frustum_planes(&view_proj_mat);
        
        // Calculate camera forward vector
        let (sy, cy) = camera.yaw.sin_cos();
        let (sp, cp) = camera.pitch.sin_cos();
        let camera_forward = Vec3::new(cy * cp, sp, sy * cp);
        
        // Frustum culling and occlusion culling: filter chunks
        let mut visible_chunks: Vec<_> = chunks.iter().filter(|chunk| {
            let min = Vec3::new(chunk.position.x, chunk.position.y, chunk.position.z);
            let max = min + Vec3::splat(crate::game::world::chunk::CHUNK_SIZE as f32);
            
            // Frustum culling
            if !Renderer::aabb_in_frustum(min, max, &frustum_planes) {
                return false;
            }
            
            // Occlusion culling
            if Renderer::is_chunk_occluded(chunk.position, crate::game::world::chunk::CHUNK_SIZE as f32, camera.position, camera_forward) {
                return false;
            }
            
            // Practical occlusion: skip if fully surrounded
            if Renderer::is_chunk_fully_surrounded(chunk, chunk_manager) {
                return false;
            }
            
            true
        }).collect();

        // Sort chunks by distance (front-to-back for better depth testing)
        visible_chunks.sort_by(|a, b| {
            let dist_a = Renderer::calculate_chunk_distance(a.position, camera.position);
            let dist_b = Renderer::calculate_chunk_distance(b.position, camera.position);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Collect all block face instances from all chunks
        let mut all_instances = Vec::new();
        for chunk in chunks {
            all_instances.extend_from_slice(&chunk.block_face_instances);
        }
        // Create instance buffer
        let instance_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BlockFace Instance Buffer"),
            contents: bytemuck::cast_slice(&all_instances),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // Static quad for a face (in local space, centered at origin, size 1)
        let quad_vertices = [
            Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], texture_index: 0 }, // bottom-left
            Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 0.0], texture_index: 0 }, // bottom-right
            Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 1.0], texture_index: 0 }, // top-right
            Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 1.0], texture_index: 0 }, // top-left
        ];
        let quad_indices = [0u16, 1, 2, 2, 3, 0];
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        {
            let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &texture.bind_group, &[]);
            for chunk in visible_chunks {
                if let Some(instance_buffer) = &chunk.instance_buffer {
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..chunk.block_face_instances.len() as u32);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
} 