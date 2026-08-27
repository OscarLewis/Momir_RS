use image::{Rgb, RgbImage};
use resvg::usvg;
use swash::{
    FontRef,
    scale::{Render, ScaleContext, Source},
    shape::ShapeContext,
    text::Script,
};
use tiny_skia::{Pixmap, Transform};
use tracing::debug;

/// Draws text onto the image with automatic word wrapping
pub(crate) fn draw_text(
    image: &mut RgbImage,
    text: &str,
    x: i32,
    baseline_y: i32,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) {
    debug!(
        text_length = text.len(),
        x, baseline_y, font_size, letter_spacing, max_width, "Drawing text"
    );

    let font = FontRef::from_index(font_data, 0).expect("invalid font");

    let mut shape_context = ShapeContext::new();
    let mut scale_context = ScaleContext::new();

    let mut scaler = scale_context.builder(font).size(font_size).build();

    let renderer = Render::new(&[Source::Outline]);

    let line_height = font_size as i32 + 5;
    let mut y = baseline_y;
    let mut line = String::new();
    let mut line_count = 0;

    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };

        let mut shaper = shape_context
            .builder(font)
            .size(font_size)
            .script(Script::Latin)
            .build();

        shaper.add_str(&candidate);

        let mut width = 0.0;

        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                width += glyph.advance + letter_spacing;
            }
        });

        if width > max_width as f32 && !line.is_empty() {
            debug!(
                line = %line,
                width,
                max_width,
                "Wrapping text line"
            );

            draw_text_line(
                image,
                &line,
                x,
                y,
                font,
                font_size,
                letter_spacing,
                &mut scaler,
                &renderer,
            );

            line_count += 1;
            y += line_height;
            line = word.to_string();
        } else {
            line = candidate;
        }
    }

    if !line.is_empty() {
        debug!(
            line = %line,
            "Rendering final text line"
        );

        draw_text_line(
            image,
            &line,
            x,
            y,
            font,
            font_size,
            letter_spacing,
            &mut scaler,
            &renderer,
        );

        line_count += 1;
    }

    debug!(line_count, final_y = y, "Finished drawing text");
}

/// Shapes and rasterizes a single line of text onto the image
pub(crate) fn draw_text_line(
    image: &mut RgbImage,
    text: &str,
    x: i32,
    baseline_y: i32,
    font: FontRef<'_>,
    font_size: f32,
    letter_spacing: f32,
    scaler: &mut swash::scale::Scaler<'_>,
    renderer: &Render,
) {
    debug!(
        text = %text,
        x,
        baseline_y,
        font_size,
        "Rendering text line"
    );

    let mut shape_context = ShapeContext::new();

    let mut shaper = shape_context
        .builder(font)
        .size(font_size)
        .script(Script::Latin)
        .build();

    shaper.add_str(text);

    let mut glyphs = Vec::new();

    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            glyphs.push((glyph.id, glyph.advance));
        }
    });

    debug!(glyph_count = glyphs.len(), "Shaped text line");

    let mut pen_x = x as f32;

    for (glyph_id, advance) in glyphs {
        if let Some(glyph_image) = renderer.render(scaler, glyph_id) {
            let placement = glyph_image.placement;

            let glyph_x = pen_x.round() as i32 + placement.left;
            let glyph_y = baseline_y - placement.top;

            debug!(
                glyph_id,
                glyph_x,
                glyph_y,
                width = placement.width,
                height = placement.height,
                "Rendering glyph"
            );

            for y in 0..placement.height {
                for x in 0..placement.width {
                    let coverage = glyph_image.data[(y * placement.width + x) as usize];

                    if coverage == 0 {
                        continue;
                    }

                    let dst_x = glyph_x + x as i32;
                    let dst_y = glyph_y + y as i32;

                    if dst_x >= 0
                        && dst_y >= 0
                        && dst_x < image.width() as i32
                        && dst_y < image.height() as i32
                    {
                        image.put_pixel(dst_x as u32, dst_y as u32, Rgb([0, 0, 0]));
                    }
                }
            }
        } else {
            debug!(glyph_id, "Failed to render glyph");
        }

        pen_x += advance + letter_spacing;
    }

    debug!(final_pen_x = pen_x, "Finished rendering text line");
}

/// Draws a black border around the card image
pub(crate) fn draw_border(image: &mut RgbImage) {
    const BORDER_WIDTH: u32 = 3;

    let width = image.width();
    let height = image.height();
    let black = Rgb([0, 0, 0]);

    debug!(
        width,
        height,
        border_width = BORDER_WIDTH,
        "Drawing card border"
    );

    for x in 0..width {
        image.put_pixel(x, 0, black);
        image.put_pixel(x, height - BORDER_WIDTH, black);
    }

    for y in 0..height {
        image.put_pixel(0, y, black);
        image.put_pixel(width - BORDER_WIDTH, y, black);
    }
}

/// Returns the rendered width of a string using the specified font and
/// letter spacing.
pub(crate) fn text_width(text: &str, font_data: &[u8], font_size: f32, letter_spacing: f32) -> f32 {
    let font = FontRef::from_index(font_data, 0).expect("invalid font");

    let mut shape_context = ShapeContext::new();

    let mut shaper = shape_context
        .builder(font)
        .size(font_size)
        .script(Script::Latin)
        .build();

    shaper.add_str(text);

    let mut width = 0.0;

    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            width += glyph.advance + letter_spacing;
        }
    });

    width
}

pub(crate) fn draw_svg(
    image: &mut RgbImage,
    svg_data: &[u8],
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the default SVG parsing options.
    let options = usvg::Options::default();

    // Parse the SVG byte data into a usvg tree.
    //
    // The tree contains the SVG's shapes, paths, text, dimensions, etc.
    let tree = usvg::Tree::from_data(svg_data, &options)?;

    // Create a pixel buffer that will hold the rendered SVG.
    //
    // Pixmap pixels are RGBA, whereas our destination RgbImage
    // contains only RGB pixels.
    let mut pixmap = Pixmap::new(width, height).ok_or("Failed to create SVG pixmap")?;

    // Calculate how much the SVG needs to be scaled horizontally
    // and vertically to fit the requested output dimensions.
    let scale_x = width as f32 / tree.size().width();
    let scale_y = height as f32 / tree.size().height();

    // Create a transform that applies the calculated scaling
    // when resvg renders the SVG.
    let transform = Transform::from_scale(scale_x, scale_y);

    // Render the SVG into the pixmap using the scale transform.
    //
    // The resulting pixmap is exactly `width × height`.
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Iterate over every pixel in the rendered SVG.
    for py in 0..height {
        for px in 0..width {
            // Get the RGBA pixel from the rendered SVG.
            let pixel = pixmap.pixel(px, py);

            if let Some(pixel) = pixel {
                // Convert the pixel's alpha value from 0..255
                // into a floating-point value from 0.0..1.0.
                //
                // 0.0 = completely transparent
                // 1.0 = completely opaque
                let alpha = pixel.alpha() as f32 / 255.0;

                // Completely transparent pixels don't change the
                // destination image, so we can skip them.
                if alpha == 0.0 {
                    continue;
                }

                // Convert the SVG-local pixel coordinates into
                // coordinates in the destination image.
                let dst_x = x + px;
                let dst_y = y + py;

                // Don't write outside the bounds of the destination image.
                //
                // This also allows the SVG to be partially outside
                // the image without causing a panic.
                if dst_x >= image.width() || dst_y >= image.height() {
                    continue;
                }

                // Get a mutable reference to the destination RGB pixel.
                let dst = image.get_pixel_mut(dst_x, dst_y);

                // Extract the RGB components from the rendered SVG pixel.
                let src_r = pixel.red() as f32;
                let src_g = pixel.green() as f32;
                let src_b = pixel.blue() as f32;

                // Alpha-composite the SVG pixel over the destination.
                //
                // For example, with alpha = 0.5:
                //
                //     output = 50% source + 50% destination
                //
                // This is done independently for each RGB channel.
                dst[0] = (src_r * alpha + dst[0] as f32 * (1.0 - alpha)) as u8;

                dst[1] = (src_g * alpha + dst[1] as f32 * (1.0 - alpha)) as u8;

                dst[2] = (src_b * alpha + dst[2] as f32 * (1.0 - alpha)) as u8;
            }
        }
    }

    Ok(())
}
