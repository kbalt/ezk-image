use crate::primitive::PrimitiveInternal;
use crate::{convert, verify_input_windows_same_size, Image};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

#[inline(never)]
#[allow(private_bounds)]
pub fn convert_multi_thread<SP: PrimitiveInternal, DP: PrimitiveInternal>(
    src: Image<&[SP]>,
    dst: Image<&mut [DP]>,
) {
    let (src_window, dst_window) = verify_input_windows_same_size(&src, &dst);

    let threads = num_cpus::get();

    if threads == 1 {
        return convert(src, dst);
    }

    let src_planes = src.planes.split(src.width, src_window, threads);
    let dst_planes = dst.planes.split(dst.width, dst_window, threads);

    src_planes.into_par_iter().zip(dst_planes).for_each(
        |((src_planes, src_window), (dst_planes, dst_window))| {
            let src = Image::new(
                src.format,
                src_planes,
                src.width,
                src_window.y + src_window.height,
                src.color,
                src.bits_per_component,
            )
            .with_window(src_window);

            let dst = Image::new(
                dst.format,
                dst_planes,
                dst.width,
                dst_window.y + dst_window.height,
                dst.color,
                dst.bits_per_component,
            )
            .with_window(dst_window);

            convert(src, dst);
        },
    );
}
