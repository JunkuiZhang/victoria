use std::rc::Rc;

pub struct FontGraphics {
    pub vertex_buffer: Rc<wgpu::Buffer>,
    pub index_buffer: Rc<wgpu::Buffer>,
    pub render_pipeline: Rc<wgpu::RenderPipeline>,
    pub uniform_bindgroup: Rc<wgpu::BindGroup>,
    pub font_data_bindgroup: Rc<wgpu::BindGroup>,
}
