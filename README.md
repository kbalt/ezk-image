
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

```rust
use ezk_image::*;

let (width, height) = (1920, 1080);

// Our source image, an RGB buffer
let rgb_image = vec![0u8; PixelFormat::RGB.buffer_size(width, height)];

// Our destination image, NV12 is a format that stores the image's luminosity and colors in the YUV space
let mut nv12_image = vec![0u8; PixelFormat::NV12.buffer_size(width, height)];

// Create the image we're converting from
let source = Image::new(
    PixelFormat::RGB,
    PixelFormatPlanes::RGB(&rgb_image[..]), // RGB only has one plane
    width,
    height,
    ColorInfo::RGB(RgbColorInfo {
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
    }),
    8, // there's 8 bits per component (R, G or B)
).unwrap();

// Create the image buffer we're converting to
let destination = Image::new(
    PixelFormat::NV12, // We're converting to NV12
    PixelFormatPlanes::infer_nv12(&mut nv12_image[..], width, height), // NV12 has 2 planes, `PixelFormatPlanes` has convenience functions to calculate them from a single buffer
    width,
    height,
    ColorInfo::YUV(YuvColorInfo {
        space: ColorSpace::BT709,
        transfer: ColorTransfer::Linear,
        primaries: ColorPrimaries::BT709,
        full_range: false,
    }),
    8, // there's 8 bits per component (Y, U or V)
).unwrap();

// Now convert the image data
convert_multi_thread(
    source,
    destination,
).unwrap();
```
