use gl::types::GLint;
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use image::RgbImage;
use rusttype::{point, Font, PositionedGlyph, Scale};

#[allow(dead_code)]
pub fn text_to_greyscale_texture_old(
    width: GLint,
    height: GLint,
    font_size: f32,
    message: &str,
    font: &Font,
) -> Result<Texture, GLErrorWrapper> {
    let scale = Scale {
        x: font_size,
        y: font_size,
    };

    let offset = point(0.0, font.v_metrics(scale).ascent);

    let glyphs: Vec<_> = font.layout(message, scale, offset).collect();

    if true {
        let width = glyphs
            .iter()
            .rev()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .next()
            .unwrap_or(0.0)
            .ceil() as usize;

        println!("width: {}, height: {}", width, font_size);
    }

    // let (width, height) = target.get_dimensions()?;
    let mut target = Texture::new()?;

    if false {
        // this doesn't work on the oculus
        let mut pixel_data = vec![99u8; (width * height) as usize];
        render_glyphs_to_grey(width, height, &glyphs, &mut pixel_data);
        target.write_pixels_and_generate_mipmap(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width,
            height,
            gl::RED,
            pixel_data.as_slice(),
        )?;
    } else {
        let mut img = RgbImage::new(width as _, height as _);
        render_glyphs_to_image(&glyphs, &mut img);

        if true {
            log::debug!(
                "text pixels {:?} .. {:?}",
                img.as_raw().iter().min(),
                img.as_raw().iter().max()
            );
        }

        target.write_pixels_and_generate_mipmap(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width,
            height,
            gl::RGB,
            img.as_raw(),
        )?;
    }
    Ok(target)
}

pub fn text_to_greyscale_texture(
    font_size: f32,
    message: &str,
    font: &Font,
) -> Result<(Texture, usize, usize), GLErrorWrapper> {
    let scale = Scale {
        x: font_size,
        y: font_size,
    };

    let metrics = font.v_metrics(scale);
    let offset = point(0.0, metrics.ascent);

    let glyphs: Vec<_> = font.layout(message, scale, offset).collect();

    let mut width = glyphs
        .iter()
        .rev()
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .next()
        .unwrap_or(0.0)
        .ceil()
        .max(1.0) as usize;

    if width & 3 != 0 {
        width = (width / 4 + 1) * 4;
    }

    let height = (metrics.ascent - metrics.descent).ceil().max(1.0) as usize;

    log::debug!(
        "height = {} = {}-{}",
        height,
        metrics.ascent,
        metrics.descent
    );

    log::debug!("width: {}, height: {}", width, height);

    // let (width, height) = target.get_dimensions()?;
    let mut target = Texture::new()?;

    if false {
        // this doesn't work on the oculus
        let mut pixel_data = vec![99u8; (width * height) as usize];
        render_glyphs_to_grey(width as i32, height as i32, &glyphs, &mut pixel_data);
        target.write_pixels_and_generate_mipmap(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width as _,
            height as _,
            gl::RED,
            pixel_data.as_slice(),
        )?;
    } else {
        let mut img = RgbImage::new(width as _, height as _);
        render_glyphs_to_image(&glyphs, &mut img);

        if true {
            log::debug!(
                "text pixels {:?} .. {:?}",
                img.as_raw().iter().min(),
                img.as_raw().iter().max()
            );
        }

        target.write_pixels_and_generate_mipmap(
            gl::TEXTURE_2D,
            0,
            gl::RGB as GLint,
            width as _,
            height as _,
            gl::RGB,
            img.as_raw(),
        )?;
    }
    Ok((target, width, height))
}

pub fn render_glyphs_to_grey<'a, 'f: 'a>(
    width: i32,
    height: i32,
    glyphs: impl IntoIterator<Item = &'a PositionedGlyph<'f>>,
    pixel_data: &mut [u8],
) {
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x0, y0, v| {
                let x = x0 as i32 + bb.min.x;
                let y = y0 as i32 + bb.min.y;
                if x >= 0 && x < width && y >= 0 && y < height {
                    let idx = x + y * width;
                    pixel_data[idx as usize] = ((1.0 - v) * 255.9) as u8;
                }
            })
        }
    }
}

#[allow(dead_code)]
pub fn render_glyphs_to_rgb<'a, 'f: 'a>(
    width: i32,
    height: i32,
    glyphs: impl IntoIterator<Item = &'a PositionedGlyph<'f>>,
    pixel_data: &mut [u8],
) {
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x0, y0, v| {
                let x = x0 as i32 + bb.min.x;
                let y = y0 as i32 + bb.min.y;
                if x >= 0 && x < width && y >= 0 && y < height {
                    let idx = (3 * (x + y * width)) as usize;
                    let a = (v * 255.9) as u8;
                    pixel_data[idx] = a;
                    pixel_data[idx + 1] = a;
                    pixel_data[idx + 2] = a;
                }
            })
        }
    }
}

pub fn render_glyphs_to_image<'a, 'f: 'a>(
    glyphs: impl IntoIterator<Item = &'a PositionedGlyph<'f>>,
    image: &mut RgbImage,
) {
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x0, y0, v| {
                let x = x0 as i32 + bb.min.x;
                let y = y0 as i32 + bb.min.y;
                if x >= 0 && y >= 0 {
                    if let Some(pix) = image.get_pixel_mut_checked(x as u32, y as u32) {
                        let a = (v * 255.9) as u8;
                        pix.0 = [a, a, a];
                    }
                }
            })
        }
    }
}
