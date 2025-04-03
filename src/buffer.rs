use core::convert::Infallible;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{Dimensions, Point, Size};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::primitives::Rectangle;

const WIDTH: usize = 84;
const HEIGHT: usize = 48;
const HEIGHT_BYTES: usize = HEIGHT / 8;
const BUF_LEN: usize = WIDTH * HEIGHT_BYTES;

pub struct Buffer {
    pub bytes: [u8; BUF_LEN],
}

impl Buffer {
    pub fn new() -> Self {
        Self { bytes: [0; BUF_LEN] }
    }
}

impl Dimensions for Buffer {
    fn bounding_box(&self) -> Rectangle {
        Rectangle {
            top_left: Point::new(0, 0),
            size: Size::new(WIDTH as u32, HEIGHT as u32),
        }
    }
}

impl DrawTarget for Buffer {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where I: IntoIterator<Item = Pixel<Self::Color>>
    {
        // FIXME: crop on bounding box
        // FIXME: we're assuming vertical bytes mode - ensure all's in order
        for Pixel(point, color) in pixels.into_iter() {
            // TODO: we're working on signed ints here - ensure all's in order
            let offset = point_to_byte_offset(&point);
            let mask = point_to_bit_mask(&point);
            // TODO: inverted vs. non-inverted mode <-> On/Off vs. white/black
            match color {
                BinaryColor::On => self.bytes[offset] |= mask,
                BinaryColor::Off => self.bytes[offset] &= !mask,
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        // TODO: inverted vs. non-inverted mode <-> On/Off vs. white/black
        self.bytes.as_mut_slice().fill(match color {
            BinaryColor::On => 0xffu8,
            BinaryColor::Off => 0x00u8,
        });
        Ok(())
    }
}

fn point_to_byte_offset(p: &Point) -> usize {
    let offset = p.y as usize / 8 * WIDTH + p.x as usize;

    // Flipped 180deg
    BUF_LEN - offset - 1
}

fn point_to_bit_mask(p: &Point) -> u8 {
    let bit_in_byte = (p.y & 7) as u8;
    // 1u8 << bit_in_byte

    // Flipped 180deg
    0x80u8 >> bit_in_byte
}
