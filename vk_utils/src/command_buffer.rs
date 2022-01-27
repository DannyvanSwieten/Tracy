use ash::vk::{
    Buffer, BufferImageCopy, ClearColorValue, CommandBuffer, CommandBufferBeginInfo, CommandPool,
    DependencyFlags, DescriptorSet, Extent2D, Extent3D, FenceCreateInfo, Framebuffer,
    ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers,
    ImageSubresourceRange, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Queue, Rect2D,
    RenderPass, RenderPassBeginInfo, SubmitInfo, SubpassContents,
};
use ash::Device;

use crate::buffer_resource::BufferResource;
use crate::graphics_pipeline::GraphicsPipeline;
use crate::image_resource::Image2DResource;
use crate::wait_handle::WaitHandle;

pub struct CommandBufferHandle {
    device: Device,
    queue: Queue,
    command_buffer: [CommandBuffer; 1],
    submit_info: [SubmitInfo; 1],
    begin_info: CommandBufferBeginInfo,
}

impl CommandBufferHandle {
    pub(crate) fn new(device: &Device, queue: &Queue, command_buffer: &CommandBuffer) -> Self {
        let mut me = Self {
            device: device.clone(),
            queue: queue.clone(),
            command_buffer: [command_buffer.clone()],
            submit_info: [SubmitInfo::default()],
            begin_info: CommandBufferBeginInfo::default(),
        };

        me.submit_info[0] = *SubmitInfo::builder().command_buffers(&me.command_buffer);
        me
    }

    pub(crate) fn begin(&self) {
        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer[0], &self.begin_info)
                .expect("Command buffer begin failed");
        }
    }

    pub(crate) fn submit(&self, command_pool: &CommandPool) -> WaitHandle {
        unsafe {
            let info = FenceCreateInfo::builder().build();
            let fence = self
                .device
                .create_fence(&info, None)
                .expect("Fence creation failed");

            self.device
                .end_command_buffer(self.command_buffer[0])
                .expect("Command Buffer end failed");
            self.device
                .queue_submit(*self.queue(), self.submit_info(), fence)
                .expect("Queue submit failed");

            WaitHandle::new(&self.device, command_pool, *self.command_buffer(), fence)
        }
    }

    pub fn bind_pipeline(&self, bind_point: PipelineBindPoint, pipeline: &ash::vk::Pipeline) {
        unsafe {
            self.device
                .cmd_bind_pipeline(*self.command_buffer(), bind_point, *pipeline);
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        layout: &PipelineLayout,
        bind_point: PipelineBindPoint,
        sets: &[DescriptorSet],
    ) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                *self.command_buffer(),
                bind_point,
                *layout,
                0,
                sets,
                &[],
            )
        }
    }

    pub fn bind_vertex_buffer(&self, first_binding: u32, buffers: &[Buffer]) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(*self.command_buffer(), first_binding, buffers, &[])
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
                *self.command_buffer(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer[0]
    }

    pub fn native_handle(&self) -> &CommandBuffer {
        &self.command_buffer[0]
    }

    pub(crate) fn submit_info(&self) -> &[SubmitInfo] {
        &self.submit_info
    }

    pub fn color_image_resource_transition(&self, image: &Image2DResource, layout: ImageLayout) {
        let barrier = ImageMemoryBarrier::builder()
            .old_layout(image.layout())
            .new_layout(layout)
            .image(*image.vk_image())
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .build();

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer[0],
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );
        }
    }

    pub fn color_image_transition(
        &self,
        image: &ash::vk::Image,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        let barrier = ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(*image)
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1)
                    .build(),
            )
            .build();

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.command_buffer[0],
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );
        }
    }

    pub fn clear_image_2d(&self, image: &Image2DResource, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            let value = ClearColorValue {
                float32: [r, g, b, a],
            };
            let range = [*ImageSubresourceRange::builder()
                .layer_count(1)
                .level_count(1)
                .aspect_mask(ImageAspectFlags::COLOR)];
            self.device.cmd_clear_color_image(
                *self.command_buffer(),
                *image.vk_image(),
                ImageLayout::GENERAL,
                &value,
                &range,
            )
        }
    }

    pub fn copy_image_2d_to_buffer(&self, image: &Image2DResource, buffer: &BufferResource) {
        let layer_info = ImageSubresourceLayers::builder()
            .layer_count(1)
            .aspect_mask(ImageAspectFlags::COLOR);
        let copy = [*BufferImageCopy::builder()
            .image_extent(
                *Extent3D::builder()
                    .width(image.width())
                    .height(image.height())
                    .depth(1),
            )
            .image_subresource(*layer_info)];

        unsafe {
            self.device.cmd_copy_image_to_buffer(
                self.command_buffer[0],
                *image.vk_image(),
                image.layout(),
                buffer.buffer,
                &copy,
            )
        }
    }

    pub(crate) fn begin_render_pass(
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
                *self.command_buffer(),
                &info,
                SubpassContents::INLINE,
            )
        }
    }

    pub(crate) fn end_render_pass(&self) {
        unsafe { self.device.cmd_end_render_pass(*self.command_buffer()) }
    }
}
