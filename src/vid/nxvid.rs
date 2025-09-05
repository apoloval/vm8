
use raylib::prelude::*;
use rand::Rng;

const SCREEN_WIDTH: i32 = 256;
const SCREEN_HEIGHT: i32 = 192;
const SCREEN_SCALE: i32 = 4;
const SCREEN_HBORDER: i32 = 32;
const SCREEN_VBORDER: i32 = 24;

const VRAM_BITPLANES: usize = 2;
const VRAM_BITPLANES_SIZE: usize = 8*1024;

const REG_BPSL: usize = 0x00;
const REG_PALR: usize = 0x40;
const REG_PALG: usize = 0x80;
const REG_PALB: usize = 0xC0;

pub struct NXVID {
    vram: Vec<Vec<u8>>,
    registers: Vec<u8>,

    rl_handle: RaylibHandle,    
    rl_thread: RaylibThread,
    rl_texture: RenderTexture2D,
    rl_texture_pixels: Vec<u8>,
}

impl NXVID {
    pub fn with_window_title(title: &str) -> Self {
        let mut rng = rand::thread_rng();

        let (mut rl_handle, rl_thread) = raylib::init()
            .size(
                (SCREEN_WIDTH+SCREEN_HBORDER*2)*SCREEN_SCALE,
                (SCREEN_HEIGHT+SCREEN_VBORDER*2)*SCREEN_SCALE,
            )
            .title(title)
            .build();
        let mut vram = Vec::new();
        for bp in 0..VRAM_BITPLANES {
            vram.push(vec![0; VRAM_BITPLANES_SIZE]);
            for i in 0..VRAM_BITPLANES_SIZE {
                vram[bp][i] = rng.gen::<u8>();
            }
        }
        let mut registers = vec![0; 256];
        for i in 0..256 {
            registers[i] = rng.gen::<u8>();
        }
        let rl_texture = rl_handle
            .load_render_texture(&rl_thread, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .unwrap();
        let rl_texture_pixels = vec![0; (SCREEN_WIDTH*SCREEN_HEIGHT*4) as usize];
        Self {vram, registers, rl_handle, rl_thread, rl_texture, rl_texture_pixels}
    }

    pub fn vram_write(&mut self, addr: u16, val: u8) {
        let bpsl = self.registers[REG_BPSL];
        self.vram[bpsl as usize][addr as usize] = val;
    }

    pub fn io_write(&mut self, port: u8, val: u8) {
        // There are 256 registers, it's safe to assume that the port is always in range
        self.registers[port as usize] = val;
    }

    pub fn refresh_screen(&mut self) {
        let mut d = self.rl_handle.begin_drawing(&self.rl_thread);
        d.clear_background(Color::BLACK);
        for y in 0..SCREEN_HEIGHT {
            let scanline = if y % 2 == 0 { 1.0f32 } else { 0.7f32 };
            for x in 0..SCREEN_WIDTH {
                let i = (y*SCREEN_WIDTH + x) as usize;

                let mut bits = 0usize;
                for bp in 0..VRAM_BITPLANES {
                    let bit = (self.vram[bp][i/8] >> (7 - (i % 8))) & 1;
                    bits |= (bit as usize) << bp;
                }
                
                let color = Color::new(
                    self.registers[REG_PALR + bits] as u8,
                    self.registers[REG_PALG + bits] as u8,
                    self.registers[REG_PALB + bits] as u8,
                    255,
                );

                let r = (color.r as f32 * scanline) as u8;
                let g = (color.g as f32 * scanline) as u8;
                let b = (color.b as f32 * scanline) as u8;
                let a = color.a;

                self.rl_texture_pixels[i*4] = r;
                self.rl_texture_pixels[i*4+1] = g;
                self.rl_texture_pixels[i*4+2] = b;
                self.rl_texture_pixels[i*4+3] = a;
                
            }
        }
        self.rl_texture.update_texture(&self.rl_texture_pixels);
        d.draw_texture_ex(
            &mut self.rl_texture,
            Vector2::new(
                (SCREEN_HBORDER*SCREEN_SCALE) as f32, 
                (SCREEN_VBORDER*SCREEN_SCALE) as f32
            ),
            0.0,
            SCREEN_SCALE as f32,
            Color::WHITE,
        );
    }
}