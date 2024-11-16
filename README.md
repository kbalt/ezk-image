
# EZK Image

ezk-image is a crate to perform conversion between common pixel formats and color spaces.

It uses SIMD and multi threading to accelerate the conversion when available, though multi-threading must
be explicitly called with [`convert_multi_thread`].

Any format can be converted to any other format.

---

## Supported Pixel formats

Bit depth of up to 16 bit per component is supported.

- I420
- I422
- I444
- I010, I012
- I210, I212
- I410, I412
- NV12
- YUYV
- RGBA, BGRA
- RGB, BGR

## Supported Color Primaries (color gamut)

- SMTPE ST 240
- BT.709 (SDR)
- BT.2020 (HDR)

## Supported Color Transfer Functions (opto-electronic transfer characteristics of samples)

- Linear
- Gamma of 2.2 and 2.8
- SRGB
- SDR (BT.601, BT.709 and BT.2020)
- BT.2100 perceptual quantization (PQ)
- BT.2100 hybrid log-gamma (HLG)

## Supported Color Spaces

- RGB
- YUV BT.601
- YUV BT.709
- YUV BT.2020
- ICtCp with perceptual quantization (PQ)
- ICtCp with hybrid log-gamma (HLG)

---

# Example

```no_run
use ezk_image::*;

let (width, height) = (1920, 1080);

// Our source image, an RGB buffer
let rgb_image = vec![0u8; PixelFormat::RGB.buffer_size(width, height)];

// Create the image we're converting from
let source = Image::from_buffer(
    PixelFormat::RGB,
    &rgb_image[..], // RGB only has one plane
    None, // No need to define strides if there's no padding between rows
    width,
    height,
    ColorInfo::RGB(RgbColorInfo {
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
    }),
).unwrap();

// Create the image buffer we're converting to
let mut destination = Image::blank(
    PixelFormat::NV12, // We're converting to NV12
    width,
    height,
    ColorInfo::YUV(YuvColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: false,
    }),
);

// Now convert the image data
convert_multi_thread(
    &source,
    &mut destination,
).unwrap();
```
