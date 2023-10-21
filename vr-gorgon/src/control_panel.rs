use crate::gorgon1::MultiGorgonSettings;
use crate::shaders::{BoxOutline, ConcentricRings, Latitude, Latitwod, SpriteRect};
use crate::text_painting;
use crate::text_painting::render_glyphs_to_image;
use crate::thumbstick_smoother::ThumbstickSmoother;
use gl::types::{GLfloat, GLsizei, GLuint};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle};
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use gl_thin::linear::{
    xr_matrix4x4f_create_scale, xr_matrix4x4f_create_translation, xr_matrix4x4f_uniform_scale,
    XrMatrix4x4f,
};
use image::{ImageBuffer, Rgb, RgbImage};
use once_cell::sync::Lazy;
use openxr_sys::Vector2f;
use rusttype::{point, Font, Point, Scale};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

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
    sprites: SpriteSheet,
    ring: BoxOutline,
    cursor: CPCursor,

    thumbstick_x_smoother: ThumbstickSmoother,
    thumbstick_y_smoother: ThumbstickSmoother,

    gorgon_val: RefCell<Option<ValueEditor>>,
}

impl ControlPanel {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let c_rings = ConcentricRings::new()?;
        let square = fab_uv_square(gpu_state, &c_rings.attributes_tuples(2))?;
        let sprite = SpriteRect::new()?;
        let latitude = Latitude::new()?;

        Ok(Self {
            c_rings,
            latitude,
            latitwod: Latitwod::new()?,
            square,
            sprites: SpriteSheet::new()?,
            sprite,
            ring: BoxOutline::new()?,
            cursor: CPCursor::default(),
            thumbstick_x_smoother: Default::default(),
            thumbstick_y_smoother: Default::default(),
            gorgon_val: RefCell::new(None),
        })
    }

    pub fn draw(
        &self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        settings: &MultiGorgonSettings,
    ) -> Result<(), GLErrorWrapper> {
        if false {
            self.sprite.draw(
                matrix,
                &[1.0; 2],
                &[0.0; 2],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        } else {
            let mut ring_sprite = None;
            let mut cursor_y = -1.0;
            cursor_y = self.header_1(matrix, gpu_state, cursor_y, &mut ring_sprite)?;

            if self.cursor.row == GorgonShape::Spiral {
                cursor_y =
                    self.spiral_menu(matrix, gpu_state, cursor_y, &mut ring_sprite, settings)?;
            }

            // if self.cursor.row == GorgonShape::Latitude {
            //     ring_y = cursor_y + 0.25;
            // }
            cursor_y = self.header_2(matrix, gpu_state, cursor_y, &mut ring_sprite)?;

            if self.cursor.row == GorgonShape::Latitude {
                cursor_y = self.latitude_menu(matrix, gpu_state, cursor_y, &mut ring_sprite)?;
            }

            // if self.cursor.row == GorgonShape::Cartesian {
            //     ring_y = cursor_y + 0.25;
            // }
            cursor_y = self.header_3(matrix, gpu_state, cursor_y, &mut ring_sprite)?;

            #[allow(unused_assignments)]
            if self.cursor.row == GorgonShape::Cartesian {
                cursor_y = self.cartesian_menu(matrix, gpu_state, cursor_y, &mut ring_sprite)?;
            }

            if let Some(sprite_loc) = ring_sprite {
                let dx = sprite_loc.offset[0];
                let dy = sprite_loc.offset[1];
                let thick = 0.06;
                let sx = sprite_loc.scale[0] / (1.0 - 2.0 * thick);
                let sy = sprite_loc.scale[1] / (1.0 - 2.0 * thick);
                let m2 = matrix
                    * xr_matrix4x4f_create_translation(dx, dy, -0.02)
                    * xr_matrix4x4f_create_scale(sx, sy, 1.0);
                self.ring.draw(&m2, &self.square, gpu_state)?;
            }
        }
        Ok(())
    }

    fn header_1<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y0: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.c_rings.draw(&m2, &self.square, gpu_state)?;
        }

        if self.cursor.row == GorgonShape::Spiral && self.cursor.subrow == GorgonParam::Enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        {
            let sprite = self.sprites.x();
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.25, y0 + sprite.scale[1], -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite.draw2(&m2, &sprite, &self.square, gpu_state)?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, y, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite
                .draw2(&m2, &self.sprites.y(), &self.square, gpu_state)?;
        }
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.75, y, -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.sprite
                .draw2(&m2, &self.sprites.z(), &self.square, gpu_state)?;
        };
        Ok(y0 + 0.5)
    }

    fn spiral_menu<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y_top: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
        settings: &MultiGorgonSettings,
    ) -> Result<f32, GLErrorWrapper> {
        let mut y = y_top;

        for (sprite, subrow) in [
            (self.sprites.freq(), GorgonParam::Frequency),
            (self.sprites.speed(), GorgonParam::Speed),
            (self.sprites.amplitude(), GorgonParam::Amplitude),
            (self.sprites.curl(), GorgonParam::Curl),
        ] {
            let x = -0.5;
            let freq_sprite = sprite;
            let y1 = y + freq_sprite.scale[1];
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y1, 0.0)
                * xr_matrix4x4f_create_scale(freq_sprite.scale[0], freq_sprite.scale[1], 1.0);
            self.sprite
                .draw2(&m2, &freq_sprite, &self.square, gpu_state)?;

            if self.cursor.row == GorgonShape::Spiral && self.cursor.subrow == subrow {
                let one = settings.lookup(self.cursor.row, self.cursor.axis);
                let val = match subrow {
                    GorgonParam::Frequency => Some(FormattableValue::U8(one.frequency)),
                    GorgonParam::Speed => Some(FormattableValue::F32(one.speed)),
                    GorgonParam::Amplitude => Some(FormattableValue::F32(one.amplitude)),
                    GorgonParam::Curl => Some(FormattableValue::F32(one.curl)),
                    _ => None,
                };

                if let Some(val) = val {
                    self.paint_editor(matrix, &freq_sprite, x, y1, val, gpu_state)?;
                }

                *ring_loc = Some(SpriteLocation::new(
                    freq_sprite.scale,
                    [x, y1],
                    &self.sprites.texture,
                ))
            }

            y += freq_sprite.scale[1] * 2.0;
        }
        Ok(y)
    }

    fn paint_editor(
        &self,
        matrix: &XrMatrix4x4f,
        sprite: &SpriteLocation,
        x: f32,
        y1: f32,
        val: FormattableValue,
        gpu_state: &mut GPUState,
    ) -> Result<(), GLErrorWrapper> {
        self.update_gorgon_val_f(val);

        if let Some(editor) = self.gorgon_val.borrow().as_ref() {
            let h = 0.25;
            let w = editor.pix_dimensions.0 as f32 * h / editor.pix_dimensions.1 as f32;
            let x2 = x + sprite.scale[0] + 0.1 + w;
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x2, y1, 0.0)
                * xr_matrix4x4f_create_scale(w, h, 1.0);
            self.sprite.draw2(
                &m2,
                &SpriteLocation::new([1.0; 2], [0.0; 2], &editor.texture),
                &self.square,
                gpu_state,
            )?;
        }
        Ok(())
    }

    fn header_2<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y0: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitude.draw(&m2, &self.square, gpu_state)?;
        }

        if self.cursor.row == GorgonShape::Latitude && self.cursor.subrow == GorgonParam::Enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, y, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(y0 + 0.5)
    }

    fn latitude_menu<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y_top: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
    ) -> Result<f32, GLErrorWrapper> {
        let mut y = y_top;

        for (sprite, subrow) in [
            (self.sprites.freq(), GorgonParam::Frequency),
            (self.sprites.speed(), GorgonParam::Speed),
            (self.sprites.amplitude(), GorgonParam::Amplitude),
            // (self.sprites.curl(), GorgonParam::Curl),
        ] {
            let x = -0.5;
            let freq_sprite = sprite;
            let y1 = y + freq_sprite.scale[1];
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y1, 0.0)
                * xr_matrix4x4f_create_scale(freq_sprite.scale[0], freq_sprite.scale[1], 1.0);
            self.sprite
                .draw2(&m2, &freq_sprite, &self.square, gpu_state)?;

            if self.cursor.row == GorgonShape::Latitude && self.cursor.subrow == subrow {
                *ring_loc = Some(SpriteLocation::new(
                    freq_sprite.scale,
                    [x, y1],
                    &self.sprites.texture,
                ))
            }

            y += freq_sprite.scale[1] * 2.0;
        }

        Ok(y)
    }

    fn header_3<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y0: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitwod.draw(&m2, &self.square, gpu_state)?;
        }

        if self.cursor.row == GorgonShape::Cartesian && self.cursor.subrow == GorgonParam::Enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(0.25, y, -0.01)
                * xr_matrix4x4f_create_scale(0.75, 0.25, 1.0);
            self.sprite.draw(
                &m2,
                &[0.75, 0.25],
                &[-0.1, 0.0],
                &self.sprites.texture,
                &self.square,
                gpu_state,
            )?;
        }
        Ok(y0 + 0.5)
    }

    fn cartesian_menu<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y_top: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
    ) -> Result<f32, GLErrorWrapper> {
        let mut y = y_top;
        for (sprite, subrow) in [
            (self.sprites.freq(), GorgonParam::Frequency),
            (self.sprites.speed(), GorgonParam::Speed),
            (self.sprites.amplitude(), GorgonParam::Amplitude),
            // (self.sprites.curl(), GorgonParam::Curl),
        ] {
            let x = -0.5;
            let freq_sprite = sprite;
            let y1 = y + freq_sprite.scale[1];
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y1, 0.0)
                * xr_matrix4x4f_create_scale(freq_sprite.scale[0], freq_sprite.scale[1], 1.0);
            self.sprite
                .draw2(&m2, &freq_sprite, &self.square, gpu_state)?;

            if self.cursor.row == GorgonShape::Cartesian && self.cursor.subrow == subrow {
                *ring_loc = Some(SpriteLocation::new(
                    freq_sprite.scale,
                    [x, y1],
                    &self.sprites.texture,
                ))
            }

            y += freq_sprite.scale[1] * 2.0;
        }

        Ok(y)
    }

    pub(crate) fn handle_thumbstick(
        &mut self,
        delta: Vector2f,
        settings: &mut MultiGorgonSettings,
    ) {
        let dx = delta.x;
        // log::debug!("thumbstick {}", dx);

        let smoothed_x = self.thumbstick_x_smoother.smooth_input(dx);
        match self.cursor.subrow {
            GorgonParam::Enable => match smoothed_x {
                Ordering::Less => self.cursor.decr_x(),
                Ordering::Equal => {}
                Ordering::Greater => self.cursor.incr_x(),
            },
            GorgonParam::Frequency => match smoothed_x {
                Ordering::Less => settings.adjust_frequency(-1, self.cursor),
                Ordering::Equal => {}
                Ordering::Greater => settings.adjust_frequency(1, self.cursor),
            },
            GorgonParam::Speed => settings.adjust_speed(dx, self.cursor),
            GorgonParam::Amplitude => settings.adjust_amplitude(dx, self.cursor),
            GorgonParam::Curl => settings.adjust_curl(dx, self.cursor),
        }

        match self.thumbstick_y_smoother.smooth_input(delta.y) {
            // yeah, this is a little backwards
            Ordering::Less => self.cursor.incr_y(),
            Ordering::Equal => {}
            Ordering::Greater => self.cursor.decr_y(),
        }
    }

    fn update_gorgon_val_f(&self, val2: FormattableValue) {
        let stale = match self.gorgon_val.borrow().as_ref() {
            Some(editor) => editor.val != val2,
            _ => true,
        };

        if stale {
            let font = default_font().unwrap();
            let msg = format!("{}", val2);
            if let Ok((texture, w, h)) = text_painting::text_to_greyscale_texture(40.0, &msg, font)
            {
                *(self.gorgon_val.borrow_mut()) =
                    Some(ValueEditor::new(val2, texture, w as _, h as _));
            }
        }
    }
}

//

#[derive(Default, PartialEq, Copy, Clone)]
pub enum GorgonShape {
    #[default]
    Spiral,
    Latitude,
    Cartesian,
}

#[derive(Default, PartialEq, Copy, Clone)]
pub enum GorgonAxis {
    #[default]
    X,
    Y,
    Z,
}

impl GorgonAxis {
    pub fn index(&self) -> u8 {
        match self {
            GorgonAxis::X => 0,
            GorgonAxis::Y => 1,
            GorgonAxis::Z => 2,
        }
    }

    pub fn x1(&self) -> f32 {
        self.index() as f32 * 0.5 - 0.25
    }
}

#[derive(Default, PartialEq, Copy, Clone)]
pub enum GorgonParam {
    #[default]
    Enable,
    Frequency,
    Speed,
    Amplitude,
    Curl,
}

#[derive(Default, Copy, Clone)]
pub struct CPCursor {
    pub row: GorgonShape,
    pub axis: GorgonAxis,
    pub subrow: GorgonParam,
}

impl CPCursor {
    pub fn incr_x(&mut self) {
        self.axis = match self.axis {
            GorgonAxis::X => GorgonAxis::Y,
            GorgonAxis::Y => GorgonAxis::Z,
            GorgonAxis::Z => GorgonAxis::X,
        };
    }

    pub fn decr_x(&mut self) {
        self.axis = match self.axis {
            GorgonAxis::X => GorgonAxis::Z,
            GorgonAxis::Y => GorgonAxis::X,
            GorgonAxis::Z => GorgonAxis::Y,
        };
    }

    pub fn incr_y(&mut self) {
        (self.row, self.subrow) = match self.row {
            GorgonShape::Spiral => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Spiral, GorgonParam::Frequency),
                GorgonParam::Frequency => (GorgonShape::Spiral, GorgonParam::Speed),
                GorgonParam::Speed => (GorgonShape::Spiral, GorgonParam::Amplitude),
                GorgonParam::Amplitude => (GorgonShape::Spiral, GorgonParam::Curl),
                GorgonParam::Curl => (GorgonShape::Latitude, GorgonParam::Enable),
            },
            GorgonShape::Latitude => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Latitude, GorgonParam::Frequency),
                GorgonParam::Frequency => (GorgonShape::Latitude, GorgonParam::Speed),
                GorgonParam::Speed => (GorgonShape::Latitude, GorgonParam::Amplitude),
                GorgonParam::Amplitude | GorgonParam::Curl => {
                    (GorgonShape::Cartesian, GorgonParam::Enable)
                }
            },

            GorgonShape::Cartesian => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Cartesian, GorgonParam::Frequency),
                GorgonParam::Frequency => (GorgonShape::Cartesian, GorgonParam::Speed),
                GorgonParam::Speed => (GorgonShape::Cartesian, GorgonParam::Amplitude),
                GorgonParam::Amplitude | GorgonParam::Curl => {
                    (GorgonShape::Spiral, GorgonParam::Enable)
                }
            },
        }
    }
    pub fn decr_y(&mut self) {
        (self.row, self.subrow) = match self.row {
            GorgonShape::Spiral => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Cartesian, GorgonParam::Amplitude),
                GorgonParam::Frequency => (GorgonShape::Spiral, GorgonParam::Enable),
                GorgonParam::Speed => (GorgonShape::Spiral, GorgonParam::Frequency),
                GorgonParam::Amplitude => (GorgonShape::Spiral, GorgonParam::Speed),
                GorgonParam::Curl => (GorgonShape::Spiral, GorgonParam::Amplitude),
            },
            GorgonShape::Latitude => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Spiral, GorgonParam::Curl),
                GorgonParam::Frequency => (GorgonShape::Latitude, GorgonParam::Enable),
                GorgonParam::Speed => (GorgonShape::Latitude, GorgonParam::Frequency),
                GorgonParam::Amplitude => (GorgonShape::Latitude, GorgonParam::Speed),
                GorgonParam::Curl => (GorgonShape::Latitude, GorgonParam::Amplitude),
            },

            GorgonShape::Cartesian => match self.subrow {
                GorgonParam::Enable => (GorgonShape::Latitude, GorgonParam::Amplitude),
                GorgonParam::Frequency => (GorgonShape::Cartesian, GorgonParam::Enable),
                GorgonParam::Speed => (GorgonShape::Cartesian, GorgonParam::Frequency),
                GorgonParam::Amplitude => (GorgonShape::Cartesian, GorgonParam::Speed),
                GorgonParam::Curl => (GorgonShape::Cartesian, GorgonParam::Amplitude),
            },
        }
    }
}

//

fn paint_text_in_image(
    font: &Font,
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    scale: f32,
    offset: Point<f32>,
    msg: &str,
) {
    let glyphs: Vec<_> = font.layout(msg, Scale::uniform(scale), offset).collect();

    render_glyphs_to_image(&glyphs, img);
}

//

pub struct SpriteLocation<'a> {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
    pub texture: &'a Texture,
}

impl<'a> SpriteLocation<'a> {
    fn new(scale: [f32; 2], offset: [f32; 2], texture: &'a Texture) -> Self {
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
        let font = default_font().unwrap();

        let width: GLsizei = 256;
        let height = 256;
        let mut img = RgbImage::new(width as _, height as _);
        let font_size = 60.0;

        let m1 = font.v_metrics(Scale::uniform(font_size));
        let ascent = m1.ascent;

        for (i, msg) in ["x", "y", "z"].iter().enumerate() {
            paint_text_in_image(
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
                paint_text_in_image(font, &mut img, small_font, point(0.0, y2), msg);
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

pub fn default_font() -> Option<&'static Font<'static>> {
    static RVAL: Lazy<Option<Font>> =
        Lazy::new(|| Font::try_from_bytes(include_bytes!("AlbertText-Bold.ttf")));

    RVAL.as_ref()
}

//

pub struct ValueEditor {
    val: FormattableValue,
    texture: Texture,
    pix_dimensions: (GLsizei, GLsizei),
}

impl ValueEditor {
    fn new(val: FormattableValue, texture: Texture, width: GLsizei, height: GLsizei) -> Self {
        Self {
            val,
            texture,
            pix_dimensions: (width, height),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum FormattableValue {
    U8(u8),
    F32(f32),
}

impl Display for FormattableValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FormattableValue::U8(val) => write!(f, "{}", val),
            FormattableValue::F32(val) => write!(f, "{:.1}", val),
        }
    }
}
