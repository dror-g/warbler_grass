use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Assets, HandleUntyped},
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

/// A raw handle which points to the shader used to render the grass.
pub(crate) const GRASS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2_263_343_952_151_597_127);

/// A raw handle to the default mesh used for grass.
///
/// The [`WarblersPlugin`] adds the corresponding mesh to the world.
/// So you should only convert the raw handle when the plugin is used
pub const GRASS_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 9_357_128_457_583_957_921);

/// A raw handle to the default normal map.
///
/// The [`WarblersPlugin`] adds the corresponding image to the world.
/// So you should only convert the raw handle when the plugin is used
pub const DEFAULT_NORMAL_MAP_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Image::TYPE_UUID, 6_322_765_653_326_473_905);

/// Adds the render pipeline for drawing grass to an [`App`]
///
/// Should always be inserted to render grass
pub struct WarblersPlugin;
impl Plugin for WarblersPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Load grass shader into cache
        load_internal_asset!(
            app,
            GRASS_SHADER_HANDLE,
            "render/assets/grass_shader.wgsl",
            Shader::from_wgsl
        );

        // Load default grass blade mesh
        let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
        meshes.set_untracked(GRASS_MESH_HANDLE, default_grass_mesh());

        // Load default normal map
        let mut images = app.world.resource_mut::<Assets<Image>>();
        images.set_untracked(DEFAULT_NORMAL_MAP_HANDLE, default_normal_map());

        app.add_systems(Update, add_dither_to_density)
            .add_asset::<DitheredBuffer>()
            .add_plugins(RenderAssetPlugin::<DitheredBuffer>::default());
        // Init resources
        app.init_resource::<GrassConfiguration>()
            .register_type::<GrassConfiguration>()
            .init_resource::<GrassNoiseTexture>();
        // Add extraction of the configuration
        app.add_plugins((
            ExtractResourcePlugin::<GrassConfiguration>::default(),
            ExtractResourcePlugin::<GrassNoiseTexture>::default(),
            ExtractComponentPlugin::<YMap>::default(),
            ExtractComponentPlugin::<NormalMap>::default(),
            ExtractComponentPlugin::<WarblerHeight>::default(),
            ExtractComponentPlugin::<GrassColor>::default(),
        ));
        // Init render app
        app.sub_app_mut(RenderApp)
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
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, queue::queue_grass_buffers.in_set(RenderSet::Queue));
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<FallbackImage>()
            .init_resource::<GrassPipeline>()
            .init_resource::<UniformBuffer>();
    }
}

/// Constructs the default normal map, which is a green image of size 1x1
///
/// Can be overridden in the corresponding [`Bundle`] using the normal_map [`Component`].
/// You can take a look at the load_grass example in the repository on how this might work
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

/// Constructs the default mesh of the grass, as shown in the examples
///
/// Can be overridden in the corresponding [`Bundle`] using the grass_mesh [`Component`].
/// You can take a look at the grass_mesh example in the repository on how this might work
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
