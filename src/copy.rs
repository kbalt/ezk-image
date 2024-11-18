use crate::{verify_input_windows, ConvertError, ImageMut, ImageRef, ImageRefExt};

#[inline(always)]
pub(crate) fn copy_impl(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
    verify_input_windows(src, dst)?;

    assert_eq!(src.format(), dst.format());

    src.bounds_check()?;
    dst.bounds_check()?;

    let desc = src.format().plane_desc();
    let planes = src.planes();
    let planes_mut = dst.planes_mut();

    for (desc, ((src_plane, src_stride), (dst_plane, dst_stride))) in
        desc.iter().zip(planes.zip(planes_mut))
    {
        let n = desc.width_op.op(src.width()) * desc.bytes_per_primitive;

        let src_rows = src_plane.chunks_exact(src_stride);
        let dst_rows = dst_plane.chunks_exact_mut(dst_stride);

        for (src_row, dst_row) in src_rows.zip(dst_rows) {
            dst_row[..n].copy_from_slice(&src_row[..n]);
        }
    }

    Ok(())
}

#[inline(never)]
#[doc(hidden)]
pub fn copy(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
    #[cfg(all(feature = "unstable", any(target_arch = "x86", target_arch = "x86_64")))]
    if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
        #[target_feature(enable = "avx512f", enable = "avx512bw")]
        unsafe fn call(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
            copy_impl(src, dst)
        }

        // Safety: Did a feature check
        unsafe {
            return call(src, dst);
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        #[target_feature(enable = "avx2")]
        unsafe fn call(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
            copy_impl(src, dst)
        }

        // Safety: Did a feature check
        unsafe {
            return call(src, dst);
        }
    }

    #[cfg(target_arch = "aarch64")]
    if crate::arch::is_aarch64_feature_detected!("neon") {
        #[target_feature(enable = "neon")]
        unsafe fn call(src: &dyn ImageRef, dst: &mut dyn ImageMut) -> Result<(), ConvertError> {
            copy_impl(src, dst)
        }

        // Safety: Did a feature check
        unsafe {
            return call(src, dst);
        }
    }

    copy_impl(src, dst)
}

#[cfg(test)]
mod tests {
    use crate::{ColorInfo, ColorPrimaries, ColorTransfer, Image, PixelFormat, YuvColorInfo};

    #[test]
    fn run_copy() {
        let width = 1920;
        let height = 1080;
        let color = ColorInfo::YUV(YuvColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            space: crate::ColorSpace::BT709,
            full_range: false,
        });

        for format in PixelFormat::variants() {
            let src = Image::blank(format, width, height, color);
            let mut dst = Image::blank(format, width, height, color);

            super::copy(&src, &mut dst).unwrap();
        }
    }

    #[test]
    fn run_copy_custom_src_strides() {
        let width = 1920;
        let height = 1080;

        let color = ColorInfo::YUV(YuvColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            space: crate::ColorSpace::BT709,
            full_range: false,
        });

        for format in PixelFormat::variants() {
            let src = Image::from_buffer(
                format,
                vec![0; format.buffer_size(width + 100, height)],
                Some(format.packed_strides(width + 100)),
                width,
                height,
                color,
            )
            .unwrap();
            let mut dst = Image::blank(format, width, height, color);

            super::copy(&src, &mut dst).unwrap();
        }
    }

    #[test]
    fn run_copy_custom_dst_strides() {
        let width = 1920;
        let height = 1080;

        let color = ColorInfo::YUV(YuvColorInfo {
            transfer: ColorTransfer::Linear,
            primaries: ColorPrimaries::BT709,
            space: crate::ColorSpace::BT709,
            full_range: false,
        });

        for format in PixelFormat::variants() {
            let src = Image::blank(format, width, height, color);

            let mut dst = Image::from_buffer(
                format,
                vec![0; format.buffer_size(width + 100, height)],
                Some(format.packed_strides(width + 100)),
                width,
                height,
                color,
            )
            .unwrap();

            super::copy(&src, &mut dst).unwrap();
        }
    }
}
