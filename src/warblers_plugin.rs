use bevy::{
    app::Plugin,
    asset::{Assets, Handle},
    core_pipeline::core_3d::Opaque3d,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        mesh::{Indices, Mesh},
        render_asset::RenderAssetPlugin,
        render_phase::AddRenderCommand,
        render_resource::{
            Extent3d, PrimitiveTopology, Shader, SpecializedMeshPipelines, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        texture::{BevyDefault, FallbackImage, ImageSampler, TextureFormatPixelInfo},
        Render, RenderApp, RenderSet,
    },
};

use crate::{
    dithering::{add_dither_to_density, DitheredBuffer},
    map::{NormalMap, YMap},
    prelude::{GrassColor, WarblerHeight},
    render::{self, cache::UniformBuffer, extract, grass_pipeline::GrassPipeline, prepare, queue},
    GrassConfiguration, GrassNoiseTexture,
};

pub struct WarblersPlugin;
impl Plugin for WarblersPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_grass_assets)
            .add_systems(Update, add_dither_to_density)
            //.add_asset::<DitheredBuffer>()
            .add_plugins(RenderAssetPlugin::<DitheredBuffer>::default())
            .init_resource::<GrassConfiguration>()
            .register_type::<GrassConfiguration>()
            .init_resource::<GrassNoiseTexture>()
            .add_plugins((
                ExtractResourcePlugin::<GrassConfiguration>::default(),
                ExtractResourcePlugin::<GrassNoiseTexture>::default(),
                ExtractComponentPlugin::<YMap>::default(),
                ExtractComponentPlugin::<NormalMap>::default(),
                ExtractComponentPlugin::<WarblerHeight>::default(),
                ExtractComponentPlugin::<GrassColor>::default(),
            ))
            .sub_app_mut(RenderApp)
            .add_render_command::<Opaque3d, render::GrassDrawCall>()
            .init_resource::<SpecializedMeshPipelines<GrassPipeline>>()
            .add_systems(
                ExtractSchedule,
                (extract::extract_grass, extract::extract_aabb),
            )
            .add_systems(
                Render,
                (
                    prepare::prepare_uniform_buffers,
                    prepare::prepare_height_buffer,
                    prepare::prepare_grass_color,
                    prepare::prepare_y_map_buffer,
                    prepare::prepare_normal_map_buffer,
                )
                .in_set(RenderSet::Queue),
            )
            .add_systems(Render, queue::queue_grass_buffers.in_set(RenderSet::Queue));
    }
}

fn setup_grass_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Load grass shader
    let grass_shader_handle = asset_server.load("render/assets/grass_shader.wgsl");

    // Load default grass blade mesh and normal map
    let grass_mesh_handle = meshes.add(default_grass_mesh());
    let normal_map_handle = images.add(default_normal_map());

    // Insert these handles into some resource for later use
    commands.insert_resource(GrassAssets {
        shader: grass_shader_handle,
        mesh: grass_mesh_handle,
        normal_map: normal_map_handle,
    });
}

#[derive(Resource)]
pub struct GrassAssets {
    pub shader: Handle<Shader>,
    mesh: Handle<Mesh>,
    normal_map: Handle<Image>,
}

fn default_normal_map() -> Image {
    let format = TextureFormat::bevy_default();
    let mut data = vec![255; format.pixel_size()];
    data[0] = 127; // R
    data[2] = 127; // B
    Image {
        data,
        texture_descriptor: TextureDescriptor {
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            format,
            dimension: TextureDimension::D2,
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler_descriptor: ImageSampler::Default,
        texture_view_descriptor: None,
    }
}

fn default_grass_mesh() -> Mesh {
    let mut grass_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    grass_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0., 0., 0.],
            [0.5, 0., 0.],
            [0.25, 0., 0.4],
            [0.25, 1., 0.15],
        ],
    );
    grass_mesh.set_indices(Some(Indices::U32(vec![1, 0, 3, 2, 1, 3, 0, 2, 3])));
    grass_mesh
}

