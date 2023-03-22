use emultendo_core::ppu::frame::Frame;
use image::RgbImage;

/// Renders frame to file.
pub fn frame_to_file(frame: &Frame, file: &str) {
    let img = RgbImage::from_raw(
        Frame::WIDTH as u32,
        Frame::HEIGHT as u32,
        frame.data().to_vec(),
    )
    .unwrap();
    img.save(file).unwrap();
}