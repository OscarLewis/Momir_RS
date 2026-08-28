use dithr::diffusion::atkinson_in_place;
use dithr::diffusion::floyd_steinberg_in_place;
use dithr::diffusion::sierra_lite_in_place;
use dithr::{QuantizeMode, gray_u8};
use image::{DynamicImage, ImageBuffer, imageops};
pub(crate) struct CardArtPipeline {}

impl CardArtPipeline {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn process(
        image: DynamicImage,
        max_width: u32,
        max_height: u32,
    ) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        let mut image = image
            .resize(max_width, max_height, imageops::FilterType::Lanczos3)
            .grayscale()
            .to_luma8();
        for pixel in image.pixels_mut() {
            pixel[0] = Self::s_curve(pixel[0]);
        }

        let width = image.width() as usize;
        let height = image.height() as usize;

        let mut buffer =
            gray_u8(image.as_mut(), width, height, width).expect("valid grayscale image");

        let quantize = QuantizeMode::gray_bits(1).expect("1-bit quantization");

        sierra_lite_in_place(&mut buffer, quantize).expect("Sierra Lite dithering failed");

        DynamicImage::ImageLuma8(image).into_rgb8()
    }

    fn s_curve(value: u8) -> u8 {
        const POINTS: &[(u8, u8)] = &[
            (0, 0),
            (32, 92),
            (64, 145),
            (96, 178),
            (128, 200),
            (160, 218),
            (192, 231),
            (224, 242),
            (255, 255),
        ];
        let value = value as f32;

        for window in POINTS.windows(2) {
            let [(x0, y0), (x1, y1)] = window else {
                unreachable!();
            };

            if value <= *x1 as f32 {
                let t = (value - *x0 as f32) / (*x1 as f32 - *x0 as f32);

                return (*y0 as f32 + t * (*y1 as f32 - *y0 as f32)).round() as u8;
            }
        }

        255
    }
}
