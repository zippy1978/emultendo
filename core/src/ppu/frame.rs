/// Render frame.
#[derive(Debug, Clone)]
pub struct Frame {
    data: Vec<u8>,
 }
 
 impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;
 
    /// Creates a new frame.
    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HEIGHT) * 3],
        }
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
 
    /// Sets pixel color in the frame.
    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
 }