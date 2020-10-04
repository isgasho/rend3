use crate::renderer::{
    shaders::{ShaderArguments, ShaderManager},
    util,
};
use shaderc::ShaderKind;
use std::{future::Future, sync::Arc};
use tracing_futures::Instrument;
use wgpu::{
    BindGroup, BindGroupLayout, Device, PipelineLayout, PipelineLayoutDescriptor, PushConstantRange, RenderPass,
    RenderPipeline, ShaderModule, ShaderStage,
};

pub struct SkyboxPass {
    pipeline: RenderPipeline,
    vertex: Arc<ShaderModule>,
    fragment: Arc<ShaderModule>,
}
impl SkyboxPass {
    pub fn new<'a>(
        device: &'a Device,
        shader_manager: &ShaderManager,
        texture_bgl: &BindGroupLayout,
        uniform_bgl: &BindGroupLayout,
    ) -> impl Future<Output = Self> + 'a {
        let new_span = tracing::warn_span!("Creating SkyboxPass");
        let new_span_guard = new_span.enter();

        let vertex = shader_manager.compile_shader(ShaderArguments {
            file: String::from("rend3/shaders/skybox.vert"),
            defines: vec![],
            kind: ShaderKind::Vertex,
            debug: cfg!(debug_assertions),
        });

        let fragment = shader_manager.compile_shader(ShaderArguments {
            file: String::from("rend3/shaders/skybox.frag"),
            defines: vec![],
            kind: ShaderKind::Fragment,
            debug: cfg!(debug_assertions),
        });

        let layout = create_skybox_pipeline_layout(device, texture_bgl, uniform_bgl);

        drop(new_span_guard);

        async move {
            let vertex = vertex.await.unwrap();
            let fragment = fragment.await.unwrap();

            let pipeline =
                util::create_render_pipeline(device, &layout, &vertex, &fragment, util::RenderPipelineType::Skybox);

            Self {
                pipeline,
                vertex,
                fragment,
            }
        }
        .instrument(new_span)
    }

    pub fn update_pipeline(&mut self, device: &Device, texture_bgl: &BindGroupLayout, uniform_bgl: &BindGroupLayout) {
        span_transfer!(_ -> update_pipeline_span, INFO, "SkyboxPass Update Pipeline");
        let layout = create_skybox_pipeline_layout(device, texture_bgl, uniform_bgl);
        let pipeline = util::create_render_pipeline(
            device,
            &layout,
            &self.vertex,
            &self.fragment,
            util::RenderPipelineType::Skybox,
        );
        self.pipeline = pipeline;
    }

    pub fn run<'a>(
        &'a self,
        rpass: &mut RenderPass<'a>,
        texture_bg: &'a BindGroup,
        uniform_bg: &'a BindGroup,
        texture: u32,
    ) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_push_constants(ShaderStage::FRAGMENT, 0, &[texture]);
        rpass.set_bind_group(0, &texture_bg, &[]);
        rpass.set_bind_group(1, &uniform_bg, &[]);
        rpass.draw(0..3, 0..1);
    }
}

fn create_skybox_pipeline_layout(
    device: &Device,
    texture_bgl: &BindGroupLayout,
    uniform_bgl: &BindGroupLayout,
) -> PipelineLayout {
    device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("skybox bgl"),
        bind_group_layouts: &[&texture_bgl, &uniform_bgl],
        push_constant_ranges: &[PushConstantRange {
            stages: ShaderStage::FRAGMENT,
            range: 0..4,
        }],
    })
}