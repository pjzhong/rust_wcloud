use ab_glyph::{point, Font, FontVec, Glyph, GlyphId, Outline, Point, PxScale, ScaleFont};
use image::{GrayImage, Luma, Pixel, Rgba, RgbaImage};

#[derive(Clone, Debug)]
pub struct GlyphData {
    pub glyphs: Vec<Glyph>,
    pub width: u32,
    pub height: u32,
}

//把文本转换为字体，方便画图
pub fn text_to_glyphs(text: &str, font: &FontVec, scale: PxScale) -> GlyphData {
    let scaled_font = font.as_scaled(scale);

    let mut glyphs: Vec<Glyph> = vec![];
    layout_paragraph(scaled_font, point(0.0, 0.0), text, &mut glyphs);

    let glyphs_height = scaled_font.height().ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs.first().unwrap().position.x;
        let last_glyph = glyphs.last().unwrap();
        let max_x = last_glyph.position.x + scaled_font.h_advance(last_glyph.id);
        (max_x - min_x).ceil() as u32
    };

    GlyphData {
        glyphs,
        width: glyphs_width,
        height: glyphs_height,
    }
}

pub fn draw_glyphs_to_gray_buffer(
    buffer: &mut GrayImage,
    glyph_data: GlyphData,
    font: &FontVec,
    point: Point,
    _rotate: bool,
) {
    for glyph in glyph_data.glyphs {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            outlined.draw(|x, y, v| {
                let (final_x, final_y) = (
                    point.x as u32 + bounds.min.x as u32 + x,
                    point.y as u32 + bounds.min.y as u32 + y,
                );
                let px = buffer.get_pixel_mut(final_x, final_y);
                *px = Luma([1])
            })
        }
    }
}

pub fn draw_glyphs_to_rgba_buffer(
    buffer: &mut RgbaImage,
    glyph_data: GlyphData,
    font: &FontVec,
    point: Point,
    _rotate: bool,
    pixel: Rgba<u8>,
) {
    for glyph in glyph_data.glyphs {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            outlined.draw(|x, y, v| {
                let (final_x, final_y) = (
                    point.x as u32 + bounds.min.x as u32 + x,
                    point.y as u32 + bounds.min.y as u32 + y,
                );
                let px = buffer.get_pixel_mut(final_x, final_y);
                px.apply2(&pixel, |old, new| {
                    ((v * new as f32) + (1.0 - v) * old as f32) as u8
                });
                if px != &Rgba::from([0; 4]) {
                    px.0[3] = 0xFF;
                }
            })
        }
    }
}

pub fn layout_paragraph<F, SF>(font: SF, position: Point, text: &str, target: &mut Vec<Glyph>)
where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<GlyphId> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                //进行换行
                caret = point(position.x, caret.y + v_advance);
            }
            continue;
        }

        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous, glyph.id);
        }
        glyph.position = caret;
        last_glyph = Some(glyph.id);
        caret.x += font.h_advance(glyph.id);

        target.push(glyph);
    }
}
