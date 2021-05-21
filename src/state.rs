use crate::easing;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenLayout {
    width: u32,
    height: u32,

    t: f32,
    section_height: f32,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub screen_layout: ScreenLayout,
    pub render_data: [[f32; 256]; 256],
}

#[derive(Debug, Clone)]
pub struct State<E: easing::Easing> {
    pub size: (u32, u32),
    pub t: f64,

    pub easing: E,
}

impl <E: easing::Easing> State<E> {
    pub fn new(width: u32, height: u32) -> Self {
        State {
            size: (width, height),
            t: 0.,
            easing: E::new_with_value(0.),
        }
    }

    pub fn step(&mut self, dt: f64) {
        self.t += dt;
        self.easing.step(dt);
    }

    pub fn get_layout(&self) -> ScreenLayout {
        let (width, height) = self.size;
        ScreenLayout {
            width, height,
            t: self.t as f32,
            section_height: self.easing.get() as f32,
        }
    }

    pub fn get_render_state(&self) -> RenderState {
        let screen_layout = self.get_layout();
        let render_data = self.get_render_data();
        RenderState {
            screen_layout,
            render_data,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size.0 = new_size.width;
        self.size.1 = new_size.height;
    }

    pub fn get_render_data(&self) -> [[f32; 256]; 256] {
        const DATA: &[u8] = include_bytes!("render/mod.rs");

        let mut out = [[0.0; 256]; 256];

        for (&first, &second) in DATA.iter().zip(DATA.iter().skip(1)) {
            out[first as usize][second as usize] += 1.0 / (DATA.len() as f32);
        }

        out
    }
}
