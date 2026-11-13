//! A small built-in example file, so the editor has something to show on first start.

/// A 16 x 8 pixel, 24-bit bitmap with a colour gradient.
pub fn sample_bmp() -> Vec<u8> {
    let w: u32 = 16;
    let h: u32 = 8;
    let row = (w * 3).div_ceil(4) * 4;
    let pixels = row * h;
    let size = 54 + pixels;
    let mut d = Vec::with_capacity(size as usize);
    d.extend_from_slice(b"BM");
    d.extend_from_slice(&size.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&54u32.to_le_bytes());
    d.extend_from_slice(&40u32.to_le_bytes());
    d.extend_from_slice(&(w as i32).to_le_bytes());
    d.extend_from_slice(&(h as i32).to_le_bytes());
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&24u16.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    d.extend_from_slice(&pixels.to_le_bytes());
    d.extend_from_slice(&2835i32.to_le_bytes());
    d.extend_from_slice(&2835i32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    d.extend_from_slice(&0u32.to_le_bytes());
    for y in 0..h {
        for x in 0..w {
            let r = (x * 255 / (w - 1)) as u8;
            let g = (y * 255 / (h - 1)) as u8;
            let b = 160u8.wrapping_sub((x * 6) as u8);
            d.extend_from_slice(&[b, g, r]);
        }
        for _ in 0..(row - w * 3) {
            d.push(0);
        }
    }
    d
}
