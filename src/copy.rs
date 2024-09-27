use crate::{
    get_and_verify_input_windows, ConvertError, Image, PixelFormat, PixelFormatPlanes, Window,
};

#[inline(always)]
#[doc(hidden)]
pub(crate) fn copy_impl(src: Image<&[u8]>, dst: Image<&mut [u8]>) -> Result<(), ConvertError> {
    get_and_verify_input_windows(&src, &dst)?;

    assert_eq!(src.format, dst.format);

    fn copy_plane(
        src: &[u8],
        src_width: usize,
        src_window: Window,

        dst: &mut [u8],
        dst_width: usize,
        dst_window: Window,
    ) {
        assert_eq!(src_window.width, dst_window.width);
        assert_eq!(src_window.height, dst_window.height);

        for y in 0..dst_window.height {
            let src_begin = (src_window.y + y) * src_width + src_window.x;
            let src_end = src_begin + src_window.width;
            let src = &src[src_begin..src_end];

            let dst_begin = (dst_window.y + y) * dst_width + dst_window.x;
            let dst_end = dst_begin + dst_window.width;
            let dst = &mut dst[dst_begin..dst_end];

            dst.copy_from_slice(src);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn copy_plane_with_divisor(
        src: &[u8],
        src_width: usize,
        src_height: usize,
        src_window: Option<Window>,

        dst: &mut [u8],
        dst_width: usize,
        dst_height: usize,
        dst_window: Option<Window>,

        div_x: usize,
        div_y: usize,
    ) {
        copy_plane(
            src,
            src_width / div_x,
            src_window
                .map(|w| Window {
                    x: w.x / div_x,
                    y: w.y / div_y,
                    width: w.width / div_x,
                    height: w.height / div_y,
                })
                .unwrap_or(Window {
                    x: 0,
                    y: 0,
                    width: src_width / div_x,
                    height: src_height / div_y,
                }),
            dst,
            dst_width / div_x,
            dst_window
                .map(|w| Window {
                    x: w.x / div_x,
                    y: w.y / div_y,
                    width: w.width / div_x,
                    height: w.height / div_y,
                })
                .unwrap_or(Window {
                    x: 0,
                    y: 0,
                    width: dst_width / div_x,
                    height: dst_height / div_y,
                }),
        );
    }

    match src.format {
        PixelFormat::I420 => {
            let PixelFormatPlanes::I420 {
                y: y_src,
                u: u_src,
                v: v_src,
            } = src.planes
            else {
                panic!()
            };
            let PixelFormatPlanes::I420 {
                y: y_dst,
                u: u_dst,
                v: v_dst,
            } = dst.planes
            else {
                panic!()
            };

            copy_plane_with_divisor(
                y_src, src.width, src.height, src.window, y_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
            copy_plane_with_divisor(
                u_src, src.width, src.height, src.window, u_dst, dst.width, dst.height, dst.window,
                2, 2,
            );
            copy_plane_with_divisor(
                v_src, src.width, src.height, src.window, v_dst, dst.width, dst.height, dst.window,
                2, 2,
            );
        }
        PixelFormat::I422 => {
            let PixelFormatPlanes::I422 {
                y: y_src,
                u: u_src,
                v: v_src,
            } = src.planes
            else {
                panic!()
            };
            let PixelFormatPlanes::I422 {
                y: y_dst,
                u: u_dst,
                v: v_dst,
            } = dst.planes
            else {
                panic!()
            };

            copy_plane_with_divisor(
                y_src, src.width, src.height, src.window, y_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
            copy_plane_with_divisor(
                u_src, src.width, src.height, src.window, u_dst, dst.width, dst.height, dst.window,
                1, 2,
            );
            copy_plane_with_divisor(
                v_src, src.width, src.height, src.window, v_dst, dst.width, dst.height, dst.window,
                1, 2,
            );
        }
        PixelFormat::I444 => {
            let PixelFormatPlanes::I444 {
                y: y_src,
                u: u_src,
                v: v_src,
            } = src.planes
            else {
                panic!()
            };
            let PixelFormatPlanes::I444 {
                y: y_dst,
                u: u_dst,
                v: v_dst,
            } = dst.planes
            else {
                panic!()
            };

            copy_plane_with_divisor(
                y_src, src.width, src.height, src.window, y_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
            copy_plane_with_divisor(
                u_src, src.width, src.height, src.window, u_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
            copy_plane_with_divisor(
                v_src, src.width, src.height, src.window, v_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
        }
        PixelFormat::NV12 => {
            let PixelFormatPlanes::NV12 {
                y: y_src,
                uv: uv_src,
            } = src.planes
            else {
                panic!()
            };
            let PixelFormatPlanes::NV12 {
                y: y_dst,
                uv: uv_dst,
            } = dst.planes
            else {
                panic!()
            };

            copy_plane_with_divisor(
                y_src, src.width, src.height, src.window, y_dst, dst.width, dst.height, dst.window,
                1, 1,
            );
            copy_plane_with_divisor(
                uv_src,
                src.width * 2,
                src.height,
                src.window,
                uv_dst,
                dst.width * 2,
                dst.height,
                dst.window,
                2,
                2,
            );
        }
        PixelFormat::YUYV => {
            let PixelFormatPlanes::YUYV(yuyv_src) = src.planes else {
                panic!()
            };
            let PixelFormatPlanes::YUYV(yuyv_dst) = dst.planes else {
                panic!()
            };

            copy_plane_with_divisor(
                yuyv_src,
                src.width * 4,
                src.height,
                src.window,
                yuyv_dst,
                dst.width * 4,
                dst.height,
                dst.window,
                2,
                1,
            );
        }
        PixelFormat::RGBA | PixelFormat::BGRA => {
            let PixelFormatPlanes::RGBA(rgba_src) = src.planes else {
                panic!()
            };
            let PixelFormatPlanes::RGBA(rgba_dst) = dst.planes else {
                panic!()
            };

            copy_plane_with_divisor(
                rgba_src,
                src.width * 4,
                src.height,
                src.window,
                rgba_dst,
                dst.width * 4,
                dst.height,
                dst.window,
                1,
                1,
            );
        }
        PixelFormat::RGB | PixelFormat::BGR => {
            let PixelFormatPlanes::RGB(rgb_src) = src.planes else {
                panic!()
            };
            let PixelFormatPlanes::RGB(rgb_dst) = dst.planes else {
                panic!()
            };

            copy_plane_with_divisor(
                rgb_src,
                src.width * 3,
                src.height,
                src.window,
                rgb_dst,
                dst.width * 3,
                dst.height,
                dst.window,
                1,
                1,
            );
        }
    }

    Ok(())
}

#[inline(never)]
pub fn copy(src: Image<&[u8]>, dst: Image<&mut [u8]>) -> Result<(), ConvertError> {
    #[cfg(all(feature = "unstable", any(target_arch = "x86", target_arch = "x86_64")))]
    if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
        #[target_feature(enable = "avx512f", enable = "avx512bw")]
        unsafe fn call(src: Image<&[u8]>, dst: Image<&mut [u8]>) -> Result<(), ConvertError> {
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
        unsafe fn call(src: Image<&[u8]>, dst: Image<&mut [u8]>) -> Result<(), ConvertError> {
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
        unsafe fn call(src: Image<&[u8]>, dst: Image<&mut [u8]>) -> Result<(), ConvertError> {
            copy_impl(src, dst)
        }

        // Safety: Did a feature check
        unsafe {
            return call(src, dst);
        }
    }

    copy_impl(src, dst)
}
