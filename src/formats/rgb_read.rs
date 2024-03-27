use super::rgb::RgbBlockVisitor;
use crate::bits::Bits;
use crate::formats::rgb::{RgbBlock, RgbPixel};
use crate::Rect;

/// Visit all pixels in an rgb image, reverse=true if its an bgr image
pub(crate) fn read_rgb_4x<const REVERSE: bool, B: Bits, Vis>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    bits_per_channel: usize,
    window: Option<Rect>,
    mut visitor: Vis,
) where
    Vis: RgbBlockVisitor,
{
    assert!((src_width * src_height * 3) <= src.len());
    assert_eq!(src_width % 2, 0);
    assert_eq!(src_height % 2, 0);

    let max_value = crate::max_value_for_bits(bits_per_channel);

    let window = window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src_width,
        height: src_height,
    });

    assert!((window.x + window.width) <= src_width);
    assert!((window.y + window.height) <= src_height);

    let src = src.as_ptr().cast::<B::Primitive>();

    // TODO: use vectors, everything is hardcoded to f32

    for y in (0..window.height).step_by(2) {
        let y = window.y + y;

        for x in (0..window.width).step_by(2) {
            let x = window.x + x;

            let rgb00 = y * src_width + x;
            let rgb01 = rgb00 + 1;

            let rgb10 = (y + 1) * src_width + x;
            let rgb11 = rgb10 + 1;

            let block = RgbBlock {
                rgb00: read_pixel::<REVERSE, B>(src, rgb00, max_value),
                rgb01: read_pixel::<REVERSE, B>(src, rgb01, max_value),
                rgb10: read_pixel::<REVERSE, B>(src, rgb10, max_value),
                rgb11: read_pixel::<REVERSE, B>(src, rgb11, max_value),
            };

            // Safety: f32 is a safe vector type, no checks needed
            unsafe {
                visitor.visit(x - window.x, y - window.y, block);
            }
        }
    }
}

fn read_pixel<const REVERSE: bool, B: Bits>(
    rgb: *const B::Primitive,
    idx: usize,
    max_value: f32,
) -> RgbPixel<f32> {
    let idx = idx * 3;

    // Safety f32 is a safe vector type
    unsafe {
        RgbPixel::from_loaded::<REVERSE>(
            B::load::<f32>(rgb.add(idx)) / max_value,
            B::load::<f32>(rgb.add(idx + 1)) / max_value,
            B::load::<f32>(rgb.add(idx + 2)) / max_value,
        )
    }
}
