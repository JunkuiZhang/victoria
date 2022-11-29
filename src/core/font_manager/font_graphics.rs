pub struct FontGraphics {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_bindgroup: wgpu::BindGroup,
    pub font_data_bindgroup: wgpu::BindGroup,
}
