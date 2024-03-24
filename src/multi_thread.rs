use crate::verify_input;
use crate::{convert, Dst, Rect, Source};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub fn convert_multi_thread<'a>(src: Source<'a>, dst: Dst<'a>) {
    let (src_window, dst_window) = verify_input(&src, &dst);

    let threads = 32;

    let src_windows = calculate_windows_by_rows(src_window, threads);
    let dst_windows = calculate_windows_by_rows(dst_window, threads);

    let src_and_dst = src_windows
        .into_iter()
        .zip(dst_windows)
        .map(|(src_window, dst_window)| {
            let src = src.with_window(src_window);
            let dst = dst
                .unsafe_copy_for_multi_threading()
                .with_window(dst_window);

            (src, dst)
        });

    src_and_dst.par_bridge().for_each(move |(src, dst)| {
        convert(src, dst);
    });
}

/// Split the work up into windows into the image by calculating the number of rows each thread should handle
fn calculate_windows_by_rows(initial_window: Rect, threads: usize) -> Vec<Rect> {
    assert_eq!(initial_window.height & 1, 0);

    let sections = initial_window.height / 2;
    let threads = threads.min(sections);

    let parts_per_section = sections / threads;
    let mut remainder = sections % threads;

    let mut rects = Vec::with_capacity(threads);

    for _ in 0..threads {
        let extra = if remainder > 0 {
            remainder -= 1;
            1
        } else {
            0
        };

        let prev = rects.last().unwrap_or(&Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });

        rects.push(Rect {
            x: 0,
            y: prev.y + prev.height,
            width: initial_window.width,
            height: (parts_per_section + extra) * 2,
        });
    }

    rects
}

#[cfg(test)]
#[test]
fn verify_windows() {
    let windows = calculate_windows_by_rows(
        Rect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1440,
        },
        32,
    );

    let mut prev = windows[0];
    let mut height_accum = prev.height;

    for rect in &windows[1..] {
        assert_eq!(rect.width, 1920);
        assert_eq!(prev.y + prev.height, rect.y);

        height_accum += rect.height;
        prev = *rect;
    }

    assert_eq!(height_accum, 1440);
}
