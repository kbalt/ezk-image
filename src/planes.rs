use crate::Rect;
use std::mem::take;

#[derive(Debug, Clone, Copy)]
pub enum PixelFormatPlanes<S: AnySlice> {
    /// See [PixelFormat::I420]
    I420 { y: S, u: S, v: S },

    /// See [PixelFormat::RGB] and [PixelFormat::BGR],
    RGB(S),

    /// See [PixelFormat::RGBA] and [PixelFormat::BGRA]
    RGBA(S),
}

impl<S: AnySlice> PixelFormatPlanes<S> {
    /// Returns if the given planes are large enough for the given dimensions
    pub fn bounds_check(&self, width: usize, height: usize) -> bool {
        let n_pixels = width * height;

        match self {
            Self::I420 { y, u, v } => {
                let uv_req_len = n_pixels / 4;

                n_pixels <= y.slice_len()
                    && uv_req_len <= u.slice_len()
                    && uv_req_len <= v.slice_len()
            }
            Self::RGB(buf) => n_pixels * 3 <= buf.slice_len(),
            Self::RGBA(buf) => n_pixels * 4 <= buf.slice_len(),
        }
    }

    pub fn infer_i420(buf: S, width: usize, height: usize) -> Self {
        let (y, tmp) = buf.slice_split_at(width * height);
        let (u, v) = tmp.slice_split_at((width * height) / 4);

        Self::I420 { y, u, v }
    }

    pub fn split(
        mut self,
        width: usize,
        initial_window: Rect,
        max_results: usize,
    ) -> Vec<(Self, Rect)> {
        assert!(width >= initial_window.x + initial_window.width);
        assert!(self.bounds_check(
            initial_window.x + initial_window.width,
            initial_window.y + initial_window.height
        ));

        let rects = calculate_windows_by_rows(initial_window, max_results);

        let mut ret = vec![];

        for rect in rects {
            match &mut self {
                Self::I420 { y, u, v } => {
                    let (y_, y_remaining) = take(y).slice_split_at(width * rect.height);
                    let (u_, u_remaining) = take(u).slice_split_at((width * rect.height) / 4);
                    let (v_, v_remaining) = take(v).slice_split_at((width * rect.height) / 4);

                    *y = y_remaining;
                    *u = u_remaining;
                    *v = v_remaining;

                    ret.push((
                        Self::I420 {
                            y: y_,
                            u: u_,
                            v: v_,
                        },
                        Rect {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::RGB(buf) => {
                    let (x, remaining) = take(buf).slice_split_at(width * 3 * rect.height);
                    *buf = remaining;
                    ret.push((
                        Self::RGB(x),
                        Rect {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
                Self::RGBA(buf) => {
                    let (x, remaining) = take(buf).slice_split_at(width * 4 * rect.height);
                    *buf = remaining;
                    ret.push((
                        Self::RGBA(x),
                        Rect {
                            x: rect.x,
                            y: 0,
                            width: rect.width,
                            height: rect.height,
                        },
                    ));
                }
            }
        }

        ret
    }
}

pub trait AnySlice: Default + Sized {
    fn slice_len(&self) -> usize;
    fn slice_split_at(self, at: usize) -> (Self, Self);
}

impl<T> AnySlice for &[T] {
    fn slice_len(&self) -> usize {
        self.len()
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at(at)
    }
}

impl<T> AnySlice for &mut [T] {
    fn slice_len(&self) -> usize {
        self.len()
    }

    fn slice_split_at(self, at: usize) -> (Self, Self) {
        self.split_at_mut(at)
    }
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

#[test]
fn xd() {
    let rgb = PixelFormatPlanes::RGB(&[0u8; 90 * 3][..]);

    let r = rgb.split(
        3,
        Rect {
            x: 0,
            y: 0,
            width: 3,
            height: 30,
        },
        9,
    );

    println!("{r:#?}")
}

#[test]
fn xd2() {
    let rgb = PixelFormatPlanes::I420 {
        y: &[0u8; 80][..],
        v: &[0u8; 20][..],
        u: &[0u8; 20][..],
    };

    let r = rgb.split(
        4,
        Rect {
            x: 0,
            y: 0,
            width: 4,
            height: 20,
        },
        4,
    );

    println!("{r:#?}")
}
