//! Deterministic Phase 0.1 basic-Latin bitmap glyph rasterizer.
//!
//! This deliberately small measurement font is repository-pinned and suitable
//! for semantic benchmark labels. It is not a future shaping or i18n stack.

const GLYPH_WIDTH: usize = 5;
const GLYPH_HEIGHT: usize = 7;

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub scale: u32,
    pub rgba: [u8; 4],
    pub letter_spacing: u32,
}

pub fn measure(text: &str, style: TextStyle) -> (u32, u32) {
    let widest = text
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0) as u32;
    let lines = text.lines().count().max(1) as u32;
    let advance = GLYPH_WIDTH as u32 * style.scale + style.letter_spacing;
    (
        widest
            .saturating_mul(advance)
            .saturating_sub(style.letter_spacing),
        lines * (GLYPH_HEIGHT as u32 + 2) * style.scale,
    )
}

pub fn draw_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    text: &str,
    style: TextStyle,
) {
    let origin_x = x;
    let mut cursor_x = x;
    let mut cursor_y = y;
    let advance = (GLYPH_WIDTH as u32 * style.scale + style.letter_spacing) as i32;
    for character in text.chars() {
        if character == '\n' {
            cursor_x = origin_x;
            cursor_y += ((GLYPH_HEIGHT as u32 + 2) * style.scale) as i32;
            continue;
        }
        let rows = glyph(character);
        for (row, bits) in rows.into_iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if bits & (1 << (GLYPH_WIDTH - 1 - column)) == 0 {
                    continue;
                }
                for sy in 0..style.scale {
                    for sx in 0..style.scale {
                        let px = cursor_x + (column as u32 * style.scale + sx) as i32;
                        let py = cursor_y + (row as u32 * style.scale + sy) as i32;
                        blend(pixels, width, height, px, py, style.rgba);
                    }
                }
            }
        }
        cursor_x += advance;
    }
}

fn blend(pixels: &mut [u8], width: u32, height: u32, x: i32, y: i32, rgba: [u8; 4]) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let index = ((y as u32 * width + x as u32) * 4) as usize;
    let alpha = f32::from(rgba[3]) / 255.0;
    for channel in 0..3 {
        pixels[index + channel] = (f32::from(rgba[channel]) * alpha
            + f32::from(pixels[index + channel]) * (1.0 - alpha))
            .round() as u8;
    }
    pixels[index + 3] = 255;
}

fn glyph(character: char) -> [u8; GLYPH_HEIGHT] {
    match character.to_ascii_uppercase() {
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0f, 0x10, 0x10, 0x10, 0x10, 0x10, 0x0f],
        'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0f, 0x10, 0x10, 0x13, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x1f],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1b, 0x11],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x0e, 0x10, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x01, 0x0e],
        '.' => [0, 0, 0, 0, 0, 0x0c, 0x0c],
        ':' => [0, 0x0c, 0x0c, 0, 0x0c, 0x0c, 0],
        '-' => [0, 0, 0, 0x0e, 0, 0, 0],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        _ => [0; GLYPH_HEIGHT],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterizes_non_rectangular_glyphs() {
        let mut pixels = vec![0_u8; 80 * 40 * 4];
        draw_rgba(
            &mut pixels,
            80,
            40,
            2,
            2,
            "A1",
            TextStyle {
                scale: 3,
                rgba: [255, 255, 255, 255],
                letter_spacing: 2,
            },
        );
        let lit = pixels.chunks_exact(4).filter(|pixel| pixel[0] > 0).count();
        assert!(lit > 40);
        assert!(
            lit < 400,
            "glyph rasterizer must not emit rectangle placeholders"
        );
    }

    #[test]
    fn measurement_handles_multiline_text() {
        let style = TextStyle {
            scale: 2,
            rgba: [0; 4],
            letter_spacing: 1,
        };
        let (width, height) = measure("AB\nC", style);
        assert_eq!(width, 21);
        assert_eq!(height, 36);
    }
}
