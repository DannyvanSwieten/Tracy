use ash::version::DeviceV1_0;
use ash::vk::{
    Buffer, CommandBuffer, DescriptorSet, Extent2D, Fence, Framebuffer, PipelineBindPoint,
    PipelineLayout, Queue, Rect2D, RenderPass, RenderPassBeginInfo, SubmitInfo, SubpassContents,
};
use ash::Device;

use crate::graphics_pipeline::GraphicsPipeline;

pub struct GraphicsCommandBuffer {
    device: Device,
    queue: Queue,
    command_buffer: [CommandBuffer; 1],
    submit_info: [SubmitInfo; 1],
    layout: PipelineLayout,
}

impl GraphicsCommandBuffer {
    pub(crate) fn new(device: &Device, queue: &Queue, command_buffer: &CommandBuffer) -> Self {
        Self {
            device: device.clone(),
            command_buffer: [command_buffer.clone()],
            queue: queue.clone(),
            submit_info: [SubmitInfo::default()],
            layout: PipelineLayout::null(),
        }
    }

    pub fn submit(&self) {
        unsafe {
            self.device
                .queue_submit(self.queue, &self.submit_info, Fence::null())
                .expect("Queue submit failed");
        }
    }

    pub fn bind_pipeline(&mut self, pipeline: &GraphicsPipeline) {
        unsafe {
            self.device.cmd_bind_pipeline(
                self.command_buffer[0],
                PipelineBindPoint::GRAPHICS,
                *pipeline.vk_handle(),
            );

            self.layout = pipeline.layout().clone()
        }
    }

    pub fn bind_descriptor_sets(&self, sets: &[DescriptorSet]) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer[0],
                PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                sets,
                &[],
            )
        }
    }

    pub fn bind_vertex_buffer(&self, first_binding: u32, buffers: &[Buffer]) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(self.command_buffer[0], first_binding, buffers, &[])
        }
    }

    pub fn draw_vertices(
        &self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.cmd_draw(
                self.command_buffer[0],
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    pub fn begin(
        &self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        width: u32,
        height: u32,
    ) {
        let info = RenderPassBeginInfo::builder()
            .render_pass(*render_pass)
            .render_area(
                Rect2D::builder()
                    .extent(Extent2D::builder().width(width).height(height).build())
                    .build(),
            )
            .framebuffer(*framebuffer)
            .build();

        unsafe {
            self.device.cmd_begin_render_pass(
                self.command_buffer[0],
                &info,
                SubpassContents::INLINE,
            )
        }
    }
}
