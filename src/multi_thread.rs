use std::mem::take;

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::{
    convert, convert_same_color_and_pixel_format, plane_decs::PlaneDesc, verify_input_windows,
    AnySlice, ConvertError, Image, ImageMut, ImageRef, StrictApi,
};

/// Parallelizes [`convert`] using as many threads as there are CPU cores.
pub fn convert_multi_thread(
    src: &impl ImageRef,
    dst: &mut impl ImageMut,
) -> Result<(), ConvertError> {
    verify_input_windows(&src, &dst)?;

    if src.format() == dst.format() && src.color() == dst.color() {
        return convert_same_color_and_pixel_format(src, dst);
    }

    let threads = num_cpus::get();

    if threads == 1 {
        return convert(src, dst);
    }

    let width = src.width();
    let height = src.height();

    let src_format = src.format();
    let dst_format = dst.format();

    let src_color = src.color();
    let dst_color = dst.color();

    let src_planes = split_planes(
        src_format.plane_desc(),
        src.planes().collect(),
        height,
        threads,
    );
    let dst_planes = split_planes(
        dst_format.plane_desc(),
        dst.planes_mut().collect(),
        height,
        threads,
    );

    src_planes.into_par_iter().zip(dst_planes).try_for_each(
        |((height, src_planes), (_, dst_planes))| {
            let src_strides = src_planes.iter().map(|p| p.1).collect();
            let src_planes = src_planes.into_iter().map(|p| p.0).collect();

            let src = Image::from_planes(
                src_format,
                src_planes,
                Some(src_strides),
                width,
                height,
                src_color,
            )
            .unwrap();

            let dst_strides = dst_planes.iter().map(|p| p.1).collect();
            let dst_planes = dst_planes.into_iter().map(|p| p.0).collect();

            let mut dst = Image::from_planes(
                dst_format,
                dst_planes,
                Some(dst_strides),
                width,
                height,
                dst_color,
            )
            .unwrap();

            convert(&src, &mut dst)
        },
    )
}

pub(crate) fn split_planes<S: AnySlice>(
    plane_decs: &[PlaneDesc],
    mut planes: Vec<(S, usize)>,
    height: usize,
    threads: usize,
) -> Vec<(usize, Vec<(S, usize)>)> {
    let sections = height / 2;
    let threads = threads.min(sections);

    let parts_per_section = sections / threads;
    let mut remainder = sections % threads;

    let mut ret = vec![];

    for _ in 0..threads {
        let extra = if remainder > 0 {
            remainder -= 1;
            1
        } else {
            0
        };

        let h = (parts_per_section + extra).strict_mul_(2);

        let mut out = vec![];

        for (plane_desc, (slice, stride)) in plane_decs.iter().zip(&mut planes) {
            let split_at = stride.strict_mul_(plane_desc.height_op.op(h));

            let (prev, rem) = take(slice).slice_split_at(split_at);
            *slice = rem;

            out.push((prev, *stride));
        }

        ret.push((h, out));
    }

    ret
}
