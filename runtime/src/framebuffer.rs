use utility::color::Color;

pub struct Framebuffer {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bytes_per_pixel: u32,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, bytes_per_pixel: u32) -> Self {
        Self {
            data: vec![0; (width * height * bytes_per_pixel) as usize],
            width,
            height,
            bytes_per_pixel,
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: &Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = ((y * self.width + x) * self.bytes_per_pixel) as usize;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = color.a;
    }

    pub fn fill(&mut self, color: Color) {
        if self.bytes_per_pixel == 3 {
            let rgb = [color.r, color.g, color.b];
            for pixel in self.data.chunks_exact_mut(self.bytes_per_pixel as usize) {
                pixel.copy_from_slice(&rgb);
            }
        } else if self.bytes_per_pixel == 4 {
            let rgba = [color.r, color.g, color.b, color.a];
            for pixel in self.data.chunks_exact_mut(self.bytes_per_pixel as usize) {
                pixel.copy_from_slice(&rgba);
            }
        } else {
            panic!("filling a framebuffer is only valid with rgb or rbga colors");
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let x = cx + dx;
                    let y = cy + dy;

                    if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                        self.set_pixel(x as u32, y as u32, &color);
                    }
                }
            }
        }
    }
}
