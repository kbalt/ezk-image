use super::{I420Block, I420Src};
use crate::bits::BitsInternal;
use crate::vector::Vector;
use crate::{PixelFormatPlanes, Rect};
use std::marker::PhantomData;

pub(crate) struct I420Reader<'a, B: BitsInternal> {
    window: Rect,

    src_width: usize,
    y: *const B::Primitive,
    u: *const B::Primitive,
    v: *const B::Primitive,

    max_value: f32,

    _m: PhantomData<&'a mut [B::Primitive]>,
    _b: PhantomData<fn() -> B>,
}

impl<'a, B: BitsInternal> I420Reader<'a, B> {
    pub(crate) fn new(
        src_width: usize,
        src_height: usize,
        src_planes: PixelFormatPlanes<&'a [B::Primitive]>,
        bits_per_component: usize,
        window: Option<Rect>,
    ) -> Self {
        let window = window.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: src_width,
            height: src_height,
        });

        assert!(src_planes.bounds_check(src_width, src_height));

        let PixelFormatPlanes::I420 { y, u, v } = src_planes else {
            panic!("Invalid PixelFormatPlanes for I420Reader");
        };

        assert!((window.x + window.width) <= src_width);
        assert!((window.y + window.height) <= src_height);

        Self {
            window,
            src_width,
            y: y.as_ptr(),
            u: u.as_ptr(),
            v: v.as_ptr(),
            max_value: crate::max_value_for_bits(bits_per_component),
            _m: PhantomData,
            _b: PhantomData,
        }
    }
}

impl<'a, B: BitsInternal> I420Src for I420Reader<'a, B> {
    #[inline(always)]
    unsafe fn read<V: Vector>(&mut self, x: usize, y: usize) -> I420Block<V> {
        let x = self.window.x + x;
        let y = self.window.y + y;

        let uv_offset = (y / 2) * (self.src_width / 2) + (x / 2);

        let y00_offset = (y * self.src_width) + x;
        let y10_offset = ((y + 1) * self.src_width) + x;

        load_and_visit_block::<V, B>(
            self.y,
            self.u,
            self.v,
            y00_offset,
            y10_offset,
            uv_offset,
            self.max_value,
        )
    }
}

#[inline(always)]
unsafe fn load_and_visit_block<V, B>(
    y_ptr: *const B::Primitive,
    u_ptr: *const B::Primitive,
    v_ptr: *const B::Primitive,
    y00_offset: usize,
    y10_offset: usize,
    uv_offset: usize,
    max_value: f32,
) -> I420Block<V>
where
    V: Vector,
    B: BitsInternal,
{
    // Load Y pixels
    let y00 = B::load::<V>(y_ptr.add(y00_offset));
    let y01 = B::load::<V>(y_ptr.add(y00_offset + V::LEN));
    let y10 = B::load::<V>(y_ptr.add(y10_offset));
    let y11 = B::load::<V>(y_ptr.add(y10_offset + V::LEN));

    // Load U and V
    let u = B::load::<V>(u_ptr.add(uv_offset));
    let v = B::load::<V>(v_ptr.add(uv_offset));

    // Convert to analog 0..=1.0
    let y00 = y00.vdivf(max_value);
    let y01 = y01.vdivf(max_value);
    let y10 = y10.vdivf(max_value);
    let y11 = y11.vdivf(max_value);

    let u = u.vdivf(max_value);
    let v = v.vdivf(max_value);

    I420Block {
        y00,
        y01,
        y10,
        y11,
        u,
        v,
    }
}
