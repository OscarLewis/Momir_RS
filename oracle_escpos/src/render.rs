use crate::layout::BorderPathLayout;
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

/// Configuration parameters for wrapping text along card boundaries.
#[derive(Clone, Copy, Debug)]
pub struct BorderWrapConfig {
    pub card_width: i32,
    pub card_height: i32,
    pub border_margin: i32,
}

impl BorderWrapConfig {
    pub fn new(card_width: i32, card_height: i32, border_margin: i32) -> Self {
        Self {
            card_width,
            card_height,
            border_margin,
        }
    }
}

/// Smoothly blends black text (0,0,0) over the background pixel using coverage alpha.
#[inline]
fn blend_pixel(dst: &mut Rgb<u8>, coverage: u8) {
    let alpha = coverage as f32 / 255.0;
    dst[0] = ((1.0 - alpha) * dst[0] as f32) as u8;
    dst[1] = ((1.0 - alpha) * dst[1] as f32) as u8;
    dst[2] = ((1.0 - alpha) * dst[2] as f32) as u8;
}

/// Renders a string wrapped counter-clockwise along perimeter paths (Left Top -> Top -> Right -> Bottom -> Left Bottom).
pub fn draw_text_around_border(
    image: &mut RgbImage,
    text: &str,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    path: &BorderPathLayout,
) {
    let font = FontRef::from_index(font_data, 0).expect("invalid font");
    let mut shape_context = ShapeContext::new();
    let mut scale_context = ScaleContext::new();
    let mut scaler = scale_context.builder(font).size(font_size).build();
    let renderer = Render::new(&[Source::Outline]);

    // Segment physical line lengths: Left Top -> Top -> Right -> Bottom -> Left Return
    let left_height = (path.left_side_bottom_y - path.left_side_top_y) as f32;
    let top_width = (path.top_end_x - path.top_start_x) as f32;
    let right_height = (path.right_side_bottom_y - path.right_side_top_y) as f32;
    let bottom_width = (path.bottom_x_start - path.bottom_x_end) as f32;
    let left_return_height = (path.left_side_return_bottom_y - path.left_side_return_top_y) as f32;

    let corner1 = left_height;
    let corner2 = corner1 + top_width;
    let corner3 = corner2 + right_height;
    let corner4 = corner3 + bottom_width;
    let max_perimeter = corner4 + left_return_height;

    struct GlyphItem {
        id: swash::GlyphId,
        advance: f32,
    }

    struct Word {
        glyphs: Vec<GlyphItem>,
        width: f32,
    }

    let mut words: Vec<Word> = Vec::new();

    for word_str in text.split_whitespace() {
        let mut shaper = shape_context
            .builder(font)
            .size(font_size)
            .script(Script::Latin)
            .build();

        shaper.add_str(word_str);

        let mut glyphs = Vec::new();
        let mut word_width = 0.0;

        shaper.shape_with(|cluster| {
            for glyph in cluster.glyphs {
                let adv = glyph.advance + letter_spacing;
                word_width += adv;
                glyphs.push(GlyphItem {
                    id: glyph.id,
                    advance: glyph.advance,
                });
            }
        });

        words.push(Word {
            glyphs,
            width: word_width,
        });
    }

    let space_width = text_width(" ", font_data, font_size, letter_spacing);
    let mut current_dist = 0.0;

    for (i, word) in words.iter().enumerate() {
        // Corner wrap transitions - leap over corners if word exceeds remaining segment space
        if current_dist < corner1 && (current_dist + word.width) > corner1 {
            current_dist = corner1; // Left -> Top
        } else if current_dist >= corner1
            && current_dist < corner2
            && (current_dist + word.width) > corner2
        {
            current_dist = corner2; // Top -> Right
        } else if current_dist >= corner2
            && current_dist < corner3
            && (current_dist + word.width) > corner3
        {
            current_dist = corner3; // Right -> Bottom
        } else if current_dist >= corner3
            && current_dist < corner4
            && (current_dist + word.width) > corner4
        {
            current_dist = corner4; // Bottom -> Left Return
        }

        for glyph in &word.glyphs {
            if current_dist >= max_perimeter {
                break;
            }

            if let Some(glyph_image) = renderer.render(&mut scaler, glyph.id) {
                if current_dist < corner1 {
                    // 1. Initial Left border (270 deg / bottom-to-top)
                    let y = path.left_side_bottom_y - current_dist as i32;
                    let x = path.left_x;
                    render_glyph_rotated_270(image, &glyph_image, x, y);
                } else if current_dist < corner2 {
                    // 2. Top border (0 deg / normal left-to-right)
                    let progress = current_dist - corner1;
                    let x = path.top_start_x + progress as i32;
                    let y = path.top_y;
                    render_glyph_normal(image, &glyph_image, x, y);
                } else if current_dist < corner3 {
                    // 3. Right border (90 deg / top-to-bottom)
                    let progress = current_dist - corner2;
                    let x = path.right_x;
                    let y = path.right_side_top_y + progress as i32;
                    render_glyph_rotated_90(image, &glyph_image, x, y);
                } else if current_dist < corner4 {
                    // 4. Bottom border (180 deg / right-to-left flipped)
                    let progress = current_dist - corner3;
                    let x = path.bottom_x_start - progress as i32;
                    let y = path.bottom_y;
                    render_glyph_rotated_180(image, &glyph_image, x, y);
                } else {
                    // 5. Left border return loop (270 deg / bottom-to-top)
                    let progress = current_dist - corner4;
                    let y = path.left_side_return_bottom_y - progress as i32;
                    let x = path.left_x;
                    render_glyph_rotated_270(image, &glyph_image, x, y);
                }
            }
            current_dist += glyph.advance + letter_spacing;
        }

        if i < words.len() - 1
            && current_dist != corner1
            && current_dist != corner2
            && current_dist != corner3
            && current_dist != corner4
        {
            current_dist += space_width;
        }
    }
}

fn render_glyph_normal(
    image: &mut RgbImage,
    glyph: &swash::scale::image::Image,
    base_x: i32,
    base_y: i32,
) {
    let placement = glyph.placement;
    let glyph_x = base_x + placement.left;
    let glyph_y = base_y - placement.top;

    for y in 0..placement.height {
        for x in 0..placement.width {
            let coverage = glyph.data[(y * placement.width + x) as usize];
            if coverage > 0 {
                let px = glyph_x + x as i32;
                let py = glyph_y + y as i32;
                if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                    blend_pixel(image.get_pixel_mut(px as u32, py as u32), coverage);
                }
            }
        }
    }
}

fn render_glyph_rotated_90(
    image: &mut RgbImage,
    glyph: &swash::scale::image::Image,
    base_x: i32,
    base_y: i32,
) {
    let placement = glyph.placement;
    for y in 0..placement.height {
        for x in 0..placement.width {
            let coverage = glyph.data[(y * placement.width + x) as usize];
            if coverage > 0 {
                let px = base_x + (placement.top - y as i32);
                let py = base_y + placement.left + x as i32;
                if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                    blend_pixel(image.get_pixel_mut(px as u32, py as u32), coverage);
                }
            }
        }
    }
}

fn render_glyph_rotated_180(
    image: &mut RgbImage,
    glyph: &swash::scale::image::Image,
    base_x: i32,
    base_y: i32,
) {
    let placement = glyph.placement;
    for y in 0..placement.height {
        for x in 0..placement.width {
            let coverage = glyph.data[(y * placement.width + x) as usize];
            if coverage > 0 {
                let px = base_x - placement.left - x as i32;
                let py = base_y + placement.top - y as i32;
                if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                    blend_pixel(image.get_pixel_mut(px as u32, py as u32), coverage);
                }
            }
        }
    }
}

fn render_glyph_rotated_270(
    image: &mut RgbImage,
    glyph: &swash::scale::image::Image,
    base_x: i32,
    base_y: i32,
) {
    let placement = glyph.placement;
    for y in 0..placement.height {
        for x in 0..placement.width {
            let coverage = glyph.data[(y * placement.width + x) as usize];
            if coverage > 0 {
                let px = base_x - (placement.top - y as i32);
                let py = base_y - placement.left - x as i32;
                if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                    blend_pixel(image.get_pixel_mut(px as u32, py as u32), coverage);
                }
            }
        }
    }
}

/// Draws a 3-pixel-wide solid black frame along the outer edges of the canvas.
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

    // Draw top and bottom outer borders
    for x in 0..width {
        image.put_pixel(x, 0, black);
        image.put_pixel(x, height - BORDER_WIDTH, black);
    }

    // Draw left and right outer borders
    for y in 0..height {
        image.put_pixel(0, y, black);
        image.put_pixel(width - BORDER_WIDTH, y, black);
    }
}

/// Renders raw SVG byte data to an offscreen pixmap and alpha-blends it
/// onto the RGB image while preserving its aspect ratio.
pub(crate) fn draw_svg(
    image: &mut RgbImage,
    svg_data: &[u8],
    x: u32,
    y: u32,
    max_width: u32,
    max_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &options)?;

    let svg_width = tree.size().width();
    let svg_height = tree.size().height();

    // Fit the SVG within the maximum dimensions while preserving
    // its aspect ratio.
    let scale = (max_width as f32 / svg_width).min(max_height as f32 / svg_height);

    let width = (svg_width * scale).round() as u32;
    let height = (svg_height * scale).round() as u32;

    let mut pixmap = Pixmap::new(width, height).ok_or("Failed to create SVG pixmap")?;

    let transform = Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Alpha blend tiny-skia pixmap pixels over the target RgbImage canvas.
    for py in 0..height {
        for px in 0..width {
            let Some(pixel) = pixmap.pixel(px, py) else {
                continue;
            };

            let alpha = pixel.alpha() as f32 / 255.0;

            if alpha == 0.0 {
                continue;
            }

            let dst_x = x + px;
            let dst_y = y + py;

            if dst_x >= image.width() || dst_y >= image.height() {
                continue;
            }

            let dst = image.get_pixel_mut(dst_x, dst_y);

            let src_r = pixel.red() as f32;
            let src_g = pixel.green() as f32;
            let src_b = pixel.blue() as f32;

            dst[0] = (src_r * alpha + dst[0] as f32 * (1.0 - alpha)) as u8;
            dst[1] = (src_g * alpha + dst[1] as f32 * (1.0 - alpha)) as u8;
            dst[2] = (src_b * alpha + dst[2] as f32 * (1.0 - alpha)) as u8;
        }
    }

    Ok(())
}

pub(crate) fn draw_svg_rotated_270(
    image: &mut RgbImage,
    svg_data: &[u8],
    x: u32,
    y: u32,
    max_width: u32,
    max_height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_data, &options)?;

    let svg_width = tree.size().width();
    let svg_height = tree.size().height();

    let scale = (max_width as f32 / svg_width).min(max_height as f32 / svg_height);

    let width = (svg_width * scale).round() as u32;
    let height = (svg_height * scale).round() as u32;

    // Rotation swaps width and height.
    let mut pixmap = Pixmap::new(height, width).ok_or("Failed to create SVG pixmap")?;

    // Rotate 270° counter-clockwise while rendering.
    let transform = Transform::from_row(0.0, -scale, scale, 0.0, 0.0, width as f32);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    for py in 0..pixmap.height() {
        for px in 0..pixmap.width() {
            let Some(pixel) = pixmap.pixel(px, py) else {
                continue;
            };

            let alpha = pixel.alpha() as f32 / 255.0;

            if alpha == 0.0 {
                continue;
            }

            let dst_x = x + px;
            let dst_y = y + py;

            if dst_x >= image.width() || dst_y >= image.height() {
                continue;
            }

            let dst = image.get_pixel_mut(dst_x, dst_y);

            let src_r = pixel.red() as f32;
            let src_g = pixel.green() as f32;
            let src_b = pixel.blue() as f32;

            dst[0] = (src_r * alpha + dst[0] as f32 * (1.0 - alpha)) as u8;
            dst[1] = (src_g * alpha + dst[1] as f32 * (1.0 - alpha)) as u8;
            dst[2] = (src_b * alpha + dst[2] as f32 * (1.0 - alpha)) as u8;
        }
    }

    Ok(())
}

/// Renders multiline text onto an image with word wrapping based on width constraints.

/// Renders multiline text onto an image with word wrapping based on width constraints,
/// respecting explicit newline ('\n') characters.
pub(crate) fn draw_text(
    image: &mut RgbImage,
    text: &str,
    x: i32,
    baseline_y: i32,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) -> i32 {
    debug!(
        text_length = text.len(),
        x, baseline_y, font_size, letter_spacing, max_width, "Drawing text"
    );

    let font = FontRef::from_index(font_data, 0).expect("invalid font");

    let mut shape_context = ShapeContext::new();
    let mut scale_context = ScaleContext::new();

    let mut scaler = scale_context.builder(font).size(font_size).build();

    let renderer = Render::new(&[Source::Outline]);

    let line_height = (font_size * 1.25).round() as i32;
    let mut y = baseline_y;
    let mut line = String::new();
    let mut line_count = 0;

    // Preserve explicit newlines by splitting on '\n' first
    for text_line in text.split('\n') {
        for word in text_line.split_whitespace() {
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

            // Wrap line if max_width is exceeded
            if width > max_width as f32 && !line.is_empty() {
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

        // Flush line at the end of explicit text line
        if !line.is_empty() {
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
            line.clear();
        } else {
            // Preserve empty lines (e.g., '\n\n')
            line_count += 1;
            y += line_height;
        }
    }

    debug!(line_count, final_y = y, "Finished drawing text");

    if line_count > 0 { y } else { baseline_y }
}

fn draw_text_line(
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

    let mut pen_x = x as f32;

    for (glyph_id, advance) in glyphs {
        if let Some(glyph_image) = renderer.render(scaler, glyph_id) {
            let placement = glyph_image.placement;
            let glyph_x = pen_x.round() as i32 + placement.left;
            let glyph_y = baseline_y - placement.top;

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
                        blend_pixel(image.get_pixel_mut(dst_x as u32, dst_y as u32), coverage);
                    }
                }
            }
        }
        pen_x += advance + letter_spacing;
    }
}

/// Calculates total rendered pixel width for a given text string, size, and letter spacing.
pub fn text_width(text: &str, font_data: &[u8], font_size: f32, letter_spacing: f32) -> f32 {
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

pub(crate) fn draw_vertical_line(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    length: i32,
    thickness: i32,
) {
    let black = Rgb([0, 0, 0]);

    for py in y..y + length {
        for px in x..x + thickness {
            if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                image.put_pixel(px as u32, py as u32, black);
            }
        }
    }
}

pub(crate) fn draw_horizontal_line(
    image: &mut RgbImage,
    x: i32,
    y: i32,
    length: i32,
    thickness: i32,
) {
    let black = Rgb([0, 0, 0]);

    for py in y..y + thickness {
        for px in x..x + length {
            if px >= 0 && py >= 0 && px < image.width() as i32 && py < image.height() as i32 {
                image.put_pixel(px as u32, py as u32, black);
            }
        }
    }
}

pub fn wrapped_line_count(
    text: &str,
    font: FontRef<'_>,
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) -> i32 {
    let mut shape_context = ShapeContext::new();
    let mut line = String::new();
    let mut line_count = 0;

    for text_line in text.split('\n') {
        for word in text_line.split_whitespace() {
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
                line_count += 1;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }

        if !line.is_empty() {
            line_count += 1;
            line.clear();
        } else {
            line_count += 1;
        }
    }

    line_count
}

/// Renders multiline text onto an image with word wrapping based on width constraints,
/// with glyphs rotated 270° counter-clockwise.
///
///
pub(crate) fn draw_text_rotated_270(
    image: &mut RgbImage,
    text: &str,
    x: i32,
    baseline_y: i32,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) -> i32 {
    let font = FontRef::from_index(font_data, 0).expect("invalid font");

    let mut shape_context = ShapeContext::new();
    let mut scale_context = ScaleContext::new();

    let mut scaler = scale_context.builder(font).size(font_size).build();

    let renderer = Render::new(&[Source::Outline]);

    // Scaling line height dynamically based on font size (1.25 ratio)
    let line_height = (font_size * 1.25).round() as i32;
    let mut line_x = x;
    let mut line = String::new();
    let mut line_count = 0;

    for text_line in text.split('\n') {
        for word in text_line.split_whitespace() {
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
                draw_text_line_rotated_270(
                    image,
                    &line,
                    line_x,
                    baseline_y,
                    font,
                    font_size,
                    letter_spacing,
                    &mut scaler,
                    &renderer,
                );

                line_count += 1;
                line_x += line_height;
                line = word.to_string();
            } else {
                line = candidate;
            }
        }

        if !line.is_empty() {
            draw_text_line_rotated_270(
                image,
                &line,
                line_x,
                baseline_y,
                font,
                font_size,
                letter_spacing,
                &mut scaler,
                &renderer,
            );

            line_count += 1;
            line_x += line_height;
            line.clear();
        } else {
            line_count += 1;
            line_x += line_height;
        }
    }

    if line_count > 0 { line_x } else { x }
}

fn draw_text_line_rotated_270(
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

    let mut pen_x: f32 = 0.0;

    for (glyph_id, advance) in glyphs {
        if let Some(glyph_image) = renderer.render(scaler, glyph_id) {
            render_glyph_rotated_270(image, &glyph_image, x, baseline_y - pen_x.round() as i32);
        }

        pen_x += advance + letter_spacing;
    }
}
/// Returns the width of the widest line after wrapping text to `max_width`.
pub fn wrapped_text_width(
    text: &str,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) -> f32 {
    let mut line = String::new();
    let mut max_line_width = 0.0_f32;

    for text_line in text.split('\n') {
        for word in text_line.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };

            let width = text_width(&candidate, font_data, font_size, letter_spacing);

            if width > max_width as f32 && !line.is_empty() {
                max_line_width =
                    max_line_width.max(text_width(&line, font_data, font_size, letter_spacing));

                line = word.to_string();
            } else {
                line = candidate;
            }
        }

        if !line.is_empty() {
            max_line_width =
                max_line_width.max(text_width(&line, font_data, font_size, letter_spacing));
            line.clear();
        }
    }

    max_line_width
}

/// Calculates the total pixel height required to render text given line wrapping and newlines.
pub fn wrapped_text_height(
    text: &str,
    font_data: &[u8],
    font_size: f32,
    letter_spacing: f32,
    max_width: i32,
) -> i32 {
    let font = FontRef::from_index(font_data, 0).expect("invalid font");
    let total_lines = wrapped_line_count(text, font, font_size, letter_spacing, max_width);
    let line_height = (font_size * 1.25).round() as i32;

    total_lines * line_height
}
