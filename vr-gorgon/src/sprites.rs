use crate::{control_panel, text_painting};
use gl::types::GLsizei;
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use image::RgbImage;
use rusttype::{point, Scale};

pub struct SpriteLocation<'a> {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
    pub texture: &'a Texture,
}

impl<'a> SpriteLocation<'a> {
    pub fn new(scale: [f32; 2], offset: [f32; 2], texture: &'a Texture) -> Self {
        Self {
            scale,
            offset,
            texture,
        }
    }

    pub fn scale(&self) -> &[f32; 2] {
        &self.scale
    }

    pub fn offset(&self) -> &[f32; 2] {
        &self.offset
    }
}

//

pub struct SpriteSheet {
    pub texture: Texture,
    pub word_ys: Vec<f32>,
}

impl SpriteSheet {
    pub fn new() -> Result<Self, GLErrorWrapper> {
        let font = control_panel::default_font().unwrap();

        let width: GLsizei = 256;
        let height = 256;
        let mut img = RgbImage::new(width as _, height as _);
        let font_size = 60.0;

        let m1 = font.v_metrics(Scale::uniform(font_size));
        let ascent = m1.ascent;

        for (i, msg) in ["x", "y", "z"].iter().enumerate() {
            text_painting::paint_text_in_image(
                font,
                &mut img,
                font_size,
                point((width * i as GLsizei) as f32 / 4.0, ascent),
                msg,
            );
        }

        let small_font = 30.0;
        let m2 = font.v_metrics(Scale::uniform(small_font));

        let word_ys: Vec<_> = ["freq", "speed", "amplitude", "curl", "kludge"]
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let y0 = height as f32 / 4.0 + 1.0 + i as f32 * 1.5 * (m2.ascent - m2.descent);
                let y2 = y0 + m2.ascent;
                text_painting::paint_text_in_image(font, &mut img, small_font, point(0.0, y2), msg);
                y0 / height as f32
            })
            .collect();

        log::debug!("word_ys {:?}", word_ys);

        // let (width, height) = target.get_dimensions()?;
        let mut target = Texture::new()?;

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
            gl::RGB as _,
            width,
            height,
            gl::RGB,
            img.as_raw(),
        )?;
        Ok(Self {
            texture: target,
            word_ys,
        })
    }

    pub fn x(&self) -> SpriteLocation<'_> {
        SpriteLocation::new([0.25; 2], [-0.1, 0.0], &self.texture)
    }

    pub fn y(&self) -> SpriteLocation {
        SpriteLocation::new([0.25; 2], [0.15, 0.0], &self.texture)
    }

    pub fn z(&self) -> SpriteLocation {
        SpriteLocation::new([0.25; 2], [0.40, 0.0], &self.texture)
    }

    pub fn freq(&self) -> SpriteLocation {
        SpriteLocation::new(
            [0.5, self.word_ys[1] - self.word_ys[0]],
            [0.0, self.word_ys[0]],
            &self.texture,
        )
    }

    pub fn speed(&self) -> SpriteLocation {
        const IDX: usize = 1;
        SpriteLocation::new(
            [0.5, self.word_ys[1 + IDX] - self.word_ys[IDX]],
            [0.0, self.word_ys[IDX]],
            &self.texture,
        )
    }
    pub fn amplitude(&self) -> SpriteLocation {
        const IDX: usize = 2;
        SpriteLocation::new(
            [0.5, self.word_ys[1 + IDX] - self.word_ys[IDX]],
            [0.0, self.word_ys[IDX]],
            &self.texture,
        )
    }
    pub fn curl(&self) -> SpriteLocation {
        const IDX: usize = 3;
        SpriteLocation::new(
            [0.5, self.word_ys[1 + IDX] - self.word_ys[IDX]],
            [0.0, self.word_ys[IDX]],
            &self.texture,
        )
    }
}
