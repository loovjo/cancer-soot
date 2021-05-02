#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderState {
    width: u32,
    height: u32,

    t: f32,
    section_height: f32,
}


pub struct State {
    pub size: (u32, u32),
    pub t: f64,

    pub section_height: f64,
}

impl State {
    pub fn new(width: u32, height: u32) -> Self {
        State {
            size: (width, height),
            t: 0.,
            section_height: 0.5,
        }
    }

    pub fn step(&mut self, dt: f64) {
        self.t += dt;
    }

    pub fn get_rs(&self) -> RenderState {
        let (width, height) = self.size;
        RenderState {
            width, height,
            t: self.t as f32,
            section_height: self.section_height as f32,
        }
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size.0 = new_size.width;
        self.size.1 = new_size.height;
    }
}