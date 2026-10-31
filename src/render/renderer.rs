use super::camera::Camera;
use super::mesher::{ChunkMesh, Mesher};
use super::pipeline::PipelineState;
use super::texture::TextureManager;
use crate::ui::draw::UiBatch;
use crate::ui::pipeline::UiPipeline;
use crate::voxel::chunk::{ChunkPos, CHUNK_DEPTH, CHUNK_WIDTH};
use crate::voxel::world::World;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GpuChunkMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct Renderer {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub pipeline_state: PipelineState,
    pub ui_pipeline: UiPipeline,
    pub chunk_meshes: FxHashMap<ChunkPos, GpuChunkMesh>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum RenderError {
    Lost,
    Outdated,
    Timeout,
    OutOfMemory,
    Other,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Force Vulkan backend
        let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_desc.backends = wgpu::Backends::VULKAN;
        let instance = wgpu::Instance::new(instance_desc);

        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find suitable Vulkan adapter");

        println!("Running on Vulkan Device: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Minecraft Vulkan Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    ..Default::default()
                },
            )
            .await
            .expect("Failed to create device");

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .expect("Surface isn't supported by the adapter");
        config.present_mode = wgpu::PresentMode::AutoNoVsync;

        surface.configure(&device, &config);

        let texture_manager = TextureManager::new(&device, &queue);
        let pipeline_state = PipelineState::new(&device, &config, &texture_manager);
        let ui_pipeline = UiPipeline::new(&device, &queue, &config, &texture_manager);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline_state,
            ui_pipeline,
            chunk_meshes: FxHashMap::default(),
        }
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.config.present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.pipeline_state.resize_depth(&self.device, new_size.width, new_size.height);
            self.ui_pipeline.resize(&self.queue, new_size.width, new_size.height);
        }
    }

    pub fn update_dirty_chunks(&mut self, world: &mut World) {
        let dirty_positions: Vec<ChunkPos> = world
            .chunks
            .iter()
            .filter_map(|(pos, chunk)| if chunk.is_dirty { Some(*pos) } else { None })
            .collect();

        if dirty_positions.is_empty() {
            return;
        }

        let new_meshes: Vec<(ChunkPos, ChunkMesh)> = dirty_positions
            .par_iter()
            .map(|&pos| {
                let chunk = &world.chunks[&pos];
                let mesh = Mesher::mesh_chunk(chunk, world);
                (pos, mesh)
            })
            .collect();

        for (pos, mesh) in new_meshes {
            if let Some(chunk) = world.chunks.get_mut(&pos) {
                chunk.is_dirty = false;
            }

            if mesh.is_empty() {
                self.chunk_meshes.remove(&pos);
                continue;
            }

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Chunk Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            self.chunk_meshes.insert(
                pos,
                GpuChunkMesh {
                    vertex_buffer,
                    index_buffer,
                    index_count: mesh.indices.len() as u32,
                },
            );
        }
    }

    pub fn render(
        &mut self,
        camera: &Camera,
        view_distance_blocks: f32,
        fog_enabled: bool,
        render_3d: bool,
        item_mesh: &(Vec<crate::render::vertex::BlockVertex>, Vec<u32>),
        ui_batch: &UiBatch,
    ) -> Result<usize, RenderError> {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost => return Err(RenderError::Lost),
            wgpu::CurrentSurfaceTexture::Outdated => return Err(RenderError::Outdated),
            wgpu::CurrentSurfaceTexture::Timeout => return Err(RenderError::Timeout),
            _ => return Err(RenderError::Other),
        };

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Update camera uniform
        let camera_uniform = camera.to_uniform(view_distance_blocks, fog_enabled);
        self.queue.write_buffer(
            &self.pipeline_state.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        // Upload UI mesh
        self.ui_pipeline.update_mesh(&self.device, ui_batch);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Vulkan Render Encoder"),
        });

        let mut rendered_chunks = 0;
        let cam_pos = camera.position;
        let cam_front = camera.front();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Vulkan Scene & UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.65,
                            g: 0.82,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.pipeline_state.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // 1. Render 3D World (if enabled)
            if render_3d {
                render_pass.set_pipeline(&self.pipeline_state.render_pipeline);
                render_pass.set_bind_group(0, &self.pipeline_state.bind_group, &[]);

                for (pos, mesh) in &self.chunk_meshes {
                    let chunk_center_x = (pos.x * CHUNK_WIDTH as i32 + (CHUNK_WIDTH as i32 / 2)) as f32;
                    let chunk_center_z = (pos.z * CHUNK_DEPTH as i32 + (CHUNK_DEPTH as i32 / 2)) as f32;

                    let to_chunk_xz = glam::Vec2::new(chunk_center_x - cam_pos.x, chunk_center_z - cam_pos.z);
                    let dist = to_chunk_xz.length();
                    if dist > 32.0 {
                        let cam_front_xz = glam::Vec2::new(cam_front.x, cam_front.z);
                        let front_len = cam_front_xz.length();
                        if front_len > 0.001 {
                            let dot = (cam_front_xz / front_len).dot(to_chunk_xz / dist);
                            if dot < -0.35 {
                                continue;
                            }
                        }
                    }

                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    rendered_chunks += 1;
                }

                // Draw 3D Dropped Items (spinning, floating mini-cubes!)
                if !item_mesh.1.is_empty() {
                    let item_vb = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Dropped Item Vertex Buffer"),
                        contents: bytemuck::cast_slice(&item_mesh.0),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                    let item_ib = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Dropped Item Index Buffer"),
                        contents: bytemuck::cast_slice(&item_mesh.1),
                        usage: wgpu::BufferUsages::INDEX,
                    });

                    render_pass.set_vertex_buffer(0, item_vb.slice(..));
                    render_pass.set_index_buffer(item_ib.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..item_mesh.1.len() as u32, 0, 0..1);
                }
            }

            // 2. Render 2D UI Overlay directly on top!
            self.ui_pipeline.render(&mut render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(surface_texture);

        Ok(rendered_chunks)
    }
}
