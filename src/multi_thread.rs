use crate::{convert, copy, get_and_verify_input_windows, Image, Primitive};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

/// Parallelizes [`convert`] using as many threads as there are CPU cores.
pub fn convert_multi_thread<SP: Primitive, DP: Primitive>(
    src: Image<&[SP]>,
    dst: Image<&mut [DP]>,
) -> Result<(), crate::ConvertError> {
    let (src_window, dst_window) = get_and_verify_input_windows(&src, &dst)?;

    if src.format == dst.format && src.color == dst.color {
        return copy(src, dst);
    }

    let threads = num_cpus::get();

    if threads == 1 {
        return convert(src, dst);
    }

    let src_planes = src.planes.split(src.width, src_window, threads);
    let dst_planes = dst.planes.split(dst.width, dst_window, threads);

    src_planes.into_par_iter().zip(dst_planes).try_for_each(
        |((src_planes, src_window), (dst_planes, dst_window))| {
            let src = Image::new(
                src.format,
                src_planes,
                src.width,
                src_window.y + src_window.height,
                src.color,
                src.bits_per_component,
            )
            .expect("Invariants have been checked")
            .with_window(src_window)
            .expect("Invariants have been checked");

            let dst = Image::new(
                dst.format,
                dst_planes,
                dst.width,
                dst_window.y + dst_window.height,
                dst.color,
                dst.bits_per_component,
            )
            .expect("Invariants have been checked")
            .with_window(dst_window)
            .expect("Invariants have been checked");

            convert(src, dst)
        },
    )
}
