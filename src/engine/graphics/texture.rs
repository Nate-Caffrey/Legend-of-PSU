use wgpu;
use image;
use log::{error, info};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

/// Helper for calculating atlas UV coordinates
pub struct AtlasUVHelper {
    atlas_size: u32,
    tile_size: f32,
}

impl AtlasUVHelper {
    pub fn new(num_textures: usize) -> Self {
        let atlas_size = (num_textures as f32).sqrt().ceil() as u32;
        let tile_size = 1.0 / atlas_size as f32;
        Self { atlas_size, tile_size }
    }

    /// Calculate UV coordinates for a specific texture in the atlas
    pub fn get_uv_coords(&self, texture_index: u32, face_uvs: [f32; 2]) -> [f32; 2] {
        let tile_x = (texture_index % self.atlas_size) as f32 * self.tile_size;
        let tile_y = (texture_index / self.atlas_size) as f32 * self.tile_size;
        
        [
            tile_x + face_uvs[0] * self.tile_size,
            tile_y + face_uvs[1] * self.tile_size,
        ]
    }

    /// Get UV coordinates for block faces based on block type and face direction
    pub fn get_block_face_uvs(&self, block_type: crate::game::world::chunk::BlockType, face_idx: usize) -> [f32; 2] {
        let texture_index = match block_type {
            crate::game::world::chunk::BlockType::Grass => match face_idx {
                4 => 0, // Top face - grass_top
                5 => 2, // Bottom face - dirt
                _ => 1, // Side faces - grass_side
            },
            crate::game::world::chunk::BlockType::Dirt => 2, // All faces - dirt
            crate::game::world::chunk::BlockType::Stone => 3, // All faces - stone
            crate::game::world::chunk::BlockType::Air => 0, // Should not happen
        };

        // Standard face UVs (will be transformed by get_uv_coords)
        let face_uvs = match face_idx {
            0 | 1 | 2 | 3 | 4 | 5 => [0.0, 1.0], // All faces use the same UV mapping
            _ => [0.0, 1.0],
        };

        self.get_uv_coords(texture_index, face_uvs)
    }
}

impl Texture {
    pub fn load_array(device: &wgpu::Device, queue: &wgpu::Queue, paths: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        if paths.is_empty() {
            error!("No texture paths provided");
            return Err("No texture paths provided".into());
        }
        let mut images = Vec::new();
        for path in paths {
            let img = image::open(path)?.to_rgba8();
            images.push(img);
        }
        let dimensions = images[0].dimensions();
        for img in &images {
            if img.dimensions() != dimensions {
                error!("All textures must have the same dimensions");
                return Err("All textures must have the same dimensions".into());
            }
        }
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: images.len() as u32,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Block Texture Array"),
            view_formats: &[],
        });
        for (i, img) in images.iter().enumerate() {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: i as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
            );
        }
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Array Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Array Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });
        Ok(Self {
            texture,
            bind_group,
            bind_group_layout,
        })
    }

    pub fn load(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();
        info!("[texture] Loaded texture: {}x{} from {}", dimensions.0, dimensions.1, path);

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });
        Ok(Self {
            texture,
            bind_group,
            bind_group_layout,
        })
    }

    pub fn create_default(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture_size = wgpu::Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Default Texture"),
            view_formats: &[],
        });

        // Create a simple checkerboard pattern
        let data = vec![
            255, 0, 0, 255,   0, 255, 0, 255,  // Red, Green
            0, 0, 255, 255,   255, 255, 255, 255,  // Blue, White
        ];

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(2),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        Self {
            texture,
            bind_group,
            bind_group_layout,
        }
    }

    /// Creates a texture atlas from individual PNG files for maximum performance
    /// This is more performant than texture arrays as it uses a single texture binding
    pub fn create_atlas_from_files(device: &wgpu::Device, queue: &wgpu::Queue, paths: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        if paths.is_empty() {
            error!("No texture paths provided for atlas");
            return Err("No texture paths provided for atlas".into());
        }

        // Load all individual images
        let mut images = Vec::new();
        for path in paths {
            let img = image::open(path)?.to_rgba8();
            images.push(img);
        }

        // Verify all textures have the same dimensions
        let tile_size = images[0].dimensions();
        for (i, img) in images.iter().enumerate() {
            if img.dimensions() != tile_size {
                error!("Texture {} has different dimensions: expected {:?}, got {:?}", 
                    paths[i], tile_size, img.dimensions());
                return Err("All textures must have the same dimensions".into());
            }
        }

        // Calculate optimal atlas layout (power of 2 for better GPU performance)
        let num_textures = images.len();
        let atlas_size = (num_textures as f32).sqrt().ceil() as u32;
        let atlas_width = atlas_size * tile_size.0;
        let atlas_height = atlas_size * tile_size.1;

        info!("Creating atlas: {}x{} with {} textures in {}x{} grid", 
            atlas_width, atlas_height, num_textures, atlas_size, atlas_size);

        // Create atlas image
        let mut atlas = image::RgbaImage::new(atlas_width, atlas_height);

        // Copy each texture to its position in the atlas
        for (i, img) in images.iter().enumerate() {
            let x = (i as u32 % atlas_size) * tile_size.0;
            let y = (i as u32 / atlas_size) * tile_size.1;
            
            // Copy the image data to the atlas
            for (src_x, src_y, pixel) in img.enumerate_pixels() {
                let dst_x = x + src_x;
                let dst_y = y + src_y;
                if dst_x < atlas_width && dst_y < atlas_height {
                    atlas.put_pixel(dst_x, dst_y, *pixel);
                }
            }
        }

        // Create wgpu texture from atlas
        let texture_size = wgpu::Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Block Texture Atlas"),
            view_formats: &[],
        });

        // Upload atlas to GPU
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * atlas_width),
                rows_per_image: Some(atlas_height),
            },
            texture_size,
        );

        // Create texture view and sampler
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group layout and bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Atlas Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        Ok(Self {
            texture,
            bind_group,
            bind_group_layout,
        })
    }
} 