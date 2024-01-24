
use raylib::prelude::*;

const SCREEN_SCALE: i32 = 4;

// Palette index register address.
const REGISTER_PAI: u8 = 1; 

pub struct NXGFX216 {
    vram: Vec<u8>,
    registers: Vec<u8>,

    rl_handle: RaylibHandle,    
    rl_thread: RaylibThread,
}

impl NXGFX216 {
    pub fn with_window_title(title: &str) -> Self {
        let (rl_handle, rl_thread) = raylib::init()
            .size(640*SCREEN_SCALE, 400*SCREEN_SCALE)
            .title(title)
            .build();
        Self {
            vram : vec![0; 2*64*1024], // 2 bitplanes of 64KB each
            registers: vec![0; 8],
            rl_handle,
            rl_thread,
        }
    }

    pub fn vram_write(&mut self, addr: u16, val: u8) {
        let bitplane = self.registers[REGISTER_PAI as usize] & 1;
        let offset = (bitplane as usize) * 64*1024 + (addr as usize);
        self.vram[offset] = val;
    }

    pub fn io_write(&mut self, port: u8, val: u8) {
        let nregs = self.registers.len();
        self.registers[port as usize % nregs] = val;
    }

    pub fn refresh_screen(&mut self) {
        let mut d = self.rl_handle.begin_drawing(&self.rl_thread);
        d.clear_background(Color::BLACK);
        for x in 0..640 {
            for y in 0..400 {
                let i = (y*640 + x) as usize;
                let b0 = (self.vram[i/8] >> (7 - (i % 8))) & 1;
                let b1 = (self.vram[64*1024 + i/8] >> (7 - (i % 8))) & 1;
                
                // TODO: obtain color from palette
                let color = match (b0, b1) {
                    (0, 0) => Color::BLACK,
                    (0, 1) => Color::BLUE,
                    (1, 0) => Color::GREEN,
                    (1, 1) => Color::WHITE,
                    _ => Color::RED,
                };

                d.draw_rectangle(SCREEN_SCALE*x, SCREEN_SCALE*y, SCREEN_SCALE, SCREEN_SCALE, color);
            }
        }
    }
}