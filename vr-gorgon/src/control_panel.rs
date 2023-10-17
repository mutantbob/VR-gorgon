use crate::shaders::{ConcentricRings, Latitude, Latitwod, SpriteRect};
use crate::text_painting::{render_glyphs_to_image, text_to_greyscale_texture};
use gl::types::{GLfloat, GLuint};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use gl_thin::linear::{
    xr_matrix4x4f_create_scale, xr_matrix4x4f_create_translation, xr_matrix4x4f_uniform_scale,
    XrMatrix4x4f,
};
use image::{ImageBuffer, Rgb, RgbImage};
use rusttype::{point, Font, Point, Scale};

pub fn fab_uv_square(
    gpu_state: &mut GPUState,
    attributes: &[(GLuint, i32, i32); 2],
) -> Result<VertexBufferBundle<'static, GLfloat, u8>, GLErrorWrapper> {
    #[rustfmt::skip]
    const XYUV: &[GLfloat] = &[
        -1.0, -1.0, 0.0, 0.0,
        1.0, -1.0, 1.0, 0.0,
        -1.0, 1.0, -0.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
    ];
    VertexBufferBundle::new(
        gpu_state,
        XYUV.into(),
        (&[0, 1, 2, 3]).into(),
        4,
        attributes,
    )
}

pub struct ControlPanel {
    c_rings: ConcentricRings,
    sprite: SpriteRect,
    latitude: Latitude,
    latitwod: Latitwod,
    square: VertexBufferBundle<'static, GLfloat, u8>,
    sprites: Texture,
}

impl ControlPanel {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let c_rings = ConcentricRings::new()?;
        let square = fab_uv_square(gpu_state, &c_rings.attributes_tuples(2))?;
        let font = Font::try_from_bytes(include_bytes!("AlbertText-Bold.ttf")).unwrap();
        let sprites = Self::make_sprite_sheet(&font)?;
        let sprite = SpriteRect::new()?;
        let latitude = Latitude::new()?;

        Ok(Self {
            c_rings,
            latitude,
            latitwod: Latitwod::new()?,
            square,
            sprites,
            sprite,
        })
    }

    fn make_sprite_sheet(font: &Font) -> Result<Texture, GLErrorWrapper> {
        let width = 128;
        let height = 128;
        let mut img = RgbImage::new(width as _, height as _);
        let font_size = 30.0;
        let scale = Scale {
            x: font_size,
            y: font_size,
        };

        let ascent = font.v_metrics(scale).ascent;

        paint_text_in_image(font, &mut img, scale, point(0.0, ascent), "x");
        paint_text_in_image(font, &mut img, scale, point(32.0, ascent), "y");
        paint_text_in_image(font, &mut img, scale, point(64.0, ascent), "z");

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
        Ok(target)
    }

    pub fn draw(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.header_1(matrix, gpu_state, -0.75)?;
        self.header_2(matrix, gpu_state, -0.25)?;
        self.header_3(matrix, gpu_state, 0.75)?;
        Ok(())
    }

    fn header_1(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.c_rings.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.25, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite.draw(
                &m2,
                &[0.25; 2],
                &[-0.1, 0.0],
                &self.sprites,
                &self.square,
                gpu_state,
            )?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite.draw(
                &m2,
                &[0.25; 2],
                &[0.15, 0.0],
                &self.sprites,
                &self.square,
                gpu_state,
            )?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.75, dy, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite.draw(
                &m2,
                &[0.25; 2],
                &[0.40, 0.0],
                &self.sprites,
                &self.square,
                gpu_state,
            )?;
        };
        Ok(())
    }

    fn header_2(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitude.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(())
    }

    fn header_3(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        dy: f32,
    ) -> Result<(), GLErrorWrapper> {
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, dy, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitwod.draw(&m2, &self.square, gpu_state)?;
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, dy, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(())
    }
}

fn paint_text_in_image(
    font: &Font,
    mut img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    scale: Scale,
    offset: Point<f32>,
    msg: &str,
) {
    let glyphs: Vec<_> = font.layout(msg, scale, offset).collect();

    render_glyphs_to_image(&glyphs, &mut img);
}
