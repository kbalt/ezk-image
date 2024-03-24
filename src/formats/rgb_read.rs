use super::rgb::RgbBlockVisitor;
use crate::formats::rgb::{RgbBlock, RgbPixel};
use crate::Rect;

/// Visit all pixels in an rgb image, reverse=true if its an bgr image
pub(crate) fn read_rgb_4x<const REVERSE: bool, Vis>(
    src_width: usize,
    src_height: usize,
    src: &[u8],
    window: Option<Rect>,
    mut visitor: Vis,
) where
    Vis: RgbBlockVisitor,
{
    assert!((src_width * src_height * 3) <= src.len());
    assert_eq!(src_width % 2, 0);
    assert_eq!(src_height % 2, 0);

    let window = window.unwrap_or(Rect {
        x: 0,
        y: 0,
        width: src_width,
        height: src_height,
    });

    assert!((window.x + window.width) <= src_width);
    assert!((window.y + window.height) <= src_height);

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
                rgb00: read_pixel::<REVERSE>(src, rgb00),
                rgb01: read_pixel::<REVERSE>(src, rgb01),
                rgb10: read_pixel::<REVERSE>(src, rgb10),
                rgb11: read_pixel::<REVERSE>(src, rgb11),
            };

            // Safety: f32 is a safe vector type, no checks needed
            unsafe {
                visitor.visit(x - window.x, y - window.y, block);
            }
        }
    }
}

fn read_pixel<const REVERSE: bool>(rgb: &[u8], idx: usize) -> RgbPixel<f32> {
    let idx = idx * 3;

    // Safety f32 is a safe vector type
    unsafe {
        RgbPixel::from_loaded8::<REVERSE>(
            f32::from(rgb[idx]),
            f32::from(rgb[idx + 1]),
            f32::from(rgb[idx + 2]),
        )
    }
}
