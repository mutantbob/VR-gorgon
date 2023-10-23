use crate::gorgon1::MultiGorgonSettings;
use crate::shaders::{BoxOutline, ConcentricRings, Latitude, Latitwod, SpriteRect};
use crate::sprites::{SpriteLocation, SpriteSheet};
use crate::text_painting;
use crate::thumbstick_smoother::ThumbstickSmoother;
use gl::types::{GLfloat, GLsizei};
use gl_thin::gl_fancy::{GPUState, VertexBufferBundle, VertexBufferLite};
use gl_thin::gl_helper::{GLErrorWrapper, Texture};
use gl_thin::linear::{
    xr_matrix4x4f_create_scale, xr_matrix4x4f_create_translation, xr_matrix4x4f_uniform_scale,
    XrMatrix4x4f,
};
use once_cell::sync::Lazy;
use openxr_sys::Vector2f;
use rusttype::Font;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

pub fn fab_uv_square_lesser(
    gpu_state: &mut GPUState,
) -> Result<VertexBufferLite<'static, GLfloat, u8>, GLErrorWrapper> {
    #[rustfmt::skip]
    const XYUV: &[GLfloat] = &[
        -1.0, -1.0, 0.0, 0.0,
        1.0, -1.0, 1.0, 0.0,
        -1.0, 1.0, -0.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
    ];
    VertexBufferLite::new(gpu_state, XYUV.into(), (&[0, 1, 2, 3]).into())
}

//

pub struct SpriteRectG {
    shader: SpriteRect,
    square: VertexBufferBundle<'static, GLfloat, u8>,
    bg: [f32; 4],
}

impl SpriteRectG {
    const FG: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    const HIGHLIGHT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

    pub fn new(
        buffers: &VertexBufferLite<'static, GLfloat, u8>,
        gpu_state: &mut GPUState,
    ) -> Result<Self, GLErrorWrapper> {
        let shader = SpriteRect::new()?;
        let square =
            VertexBufferBundle::from_buffers(gpu_state, buffers, 4, &shader.attributes_tuples(2))?;
        Ok(Self {
            shader,
            square,
            bg: [1.0, 1.0, 1.0, 0.5],
        })
    }

    pub fn draw(
        &self,
        matrix: &XrMatrix4x4f,
        scale: &[f32; 2],
        offset: &[f32; 2],
        texture: &Texture,
        gpu_state: &mut GPUState,
        fg: &[f32; 4],
    ) -> Result<(), GLErrorWrapper> {
        self.shader.draw(
            matrix,
            scale,
            offset,
            texture,
            fg,
            &self.bg,
            &self.square,
            gpu_state,
        )
    }

    pub fn draw2(
        &self,
        matrix: &XrMatrix4x4f,
        sprite: &SpriteLocation,
        gpu_state: &mut GPUState,
        fg: &[f32; 4],
    ) -> Result<(), GLErrorWrapper> {
        let bg = &self.bg;
        self.shader
            .draw2(matrix, sprite, fg, bg, &self.square, gpu_state)
    }

    pub fn fg_for(highlight: bool) -> &'static [f32; 4] {
        if highlight {
            &Self::HIGHLIGHT
        } else {
            &Self::FG
        }
    }
}

//

macro_rules! shader_plus_geometry {
    ($st:ident, $stg:ident) => {
        pub struct $stg {
            shader: $st,
            square: VertexBufferBundle<'static, GLfloat, u8>,
        }

        impl $stg {
            pub fn new(
                buffers: &VertexBufferLite<'static, GLfloat, u8>,
                gpu_state: &mut GPUState,
            ) -> Result<Self, GLErrorWrapper> {
                let shader = $st::new()?;
                let square = VertexBufferBundle::from_buffers(
                    gpu_state,
                    buffers,
                    4,
                    &shader.attributes_tuples(2),
                )?;
                Ok(Self { shader, square })
            }

            pub fn draw(
                &self,
                matrix: &XrMatrix4x4f,
                gpu_state: &mut GPUState,
            ) -> Result<(), GLErrorWrapper> {
                self.shader.draw(matrix, &self.square, gpu_state)
            }
        }
    };
}

shader_plus_geometry! {ConcentricRings, ConcentricRingsG}
shader_plus_geometry! {BoxOutline, BoxOutlineG}
shader_plus_geometry! {Latitude, LatitudeG}
shader_plus_geometry! {Latitwod, LatitwodG}

//

pub struct ControlPanel {
    c_rings: ConcentricRingsG,
    sprite: SpriteRectG,
    latitude: LatitudeG,
    latitwod: LatitwodG,
    sprites: SpriteSheet,
    ring: BoxOutlineG,
    cursor: CPCursor,

    thumbstick_x_smoother: ThumbstickSmoother,
    thumbstick_y_smoother: ThumbstickSmoother,

    gorgon_val: RefCell<Option<ValueEditor>>,
}

impl ControlPanel {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let square = fab_uv_square_lesser(gpu_state)?;
        let c_rings = ConcentricRingsG::new(&square, gpu_state)?;
        let sprite = SpriteRectG::new(&square, gpu_state)?;
        let latitude = LatitudeG::new(&square, gpu_state)?;

        // let square = fab_uv_square(gpu_state, &c_rings.attributes_tuples(2))?;

        Ok(Self {
            c_rings,
            latitude,
            latitwod: LatitwodG::new(&square, gpu_state)?,
            // square,
            sprites: SpriteSheet::new(gpu_state)?,
            sprite,
            ring: BoxOutlineG::new(&square, gpu_state)?,
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
        let mut ring_sprite = None;
        let mut cursor_y = -1.0;
        cursor_y = self.header_1(matrix, gpu_state, cursor_y, &mut ring_sprite, settings)?;

        if self.cursor.row == GorgonShape::Spiral {
            cursor_y = self.spiral_menu(matrix, gpu_state, cursor_y, &mut ring_sprite, settings)?;
        }

        // if self.cursor.row == GorgonShape::Latitude {
        //     ring_y = cursor_y + 0.25;
        // }
        cursor_y = self.header_2(matrix, gpu_state, cursor_y, &mut ring_sprite, settings)?;

        if self.cursor.row == GorgonShape::Latitude {
            cursor_y = self.latitude_menu(matrix, gpu_state, cursor_y, &mut ring_sprite)?;
        }

        // if self.cursor.row == GorgonShape::Cartesian {
        //     ring_y = cursor_y + 0.25;
        // }
        cursor_y = self.header_3(matrix, gpu_state, cursor_y, &mut ring_sprite, settings)?;

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
            self.ring.draw(&m2, gpu_state)?;
        }
        Ok(())
    }

    fn header_1<'a>(
        &'a self,
        matrix: &XrMatrix4x4f,
        gpu_state: &mut GPUState,
        y0: f32,
        ring_loc: &mut Option<SpriteLocation<'a>>,
        settings: &MultiGorgonSettings,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.c_rings.draw(&m2, gpu_state)?;
        }

        let spiral_enable =
            self.cursor.row == GorgonShape::Spiral && self.cursor.subrow == GorgonParam::Enable;
        if spiral_enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        for (i, (sprite, axis)) in [
            (self.sprites.x(), GorgonAxis::X),
            (self.sprites.y(), GorgonAxis::Y),
            (self.sprites.z(), GorgonAxis::Z),
        ]
        .iter()
        .enumerate()
        {
            let x = -0.25 + i as f32 * 0.5;
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y0 + sprite.scale[1], -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            let fg = if settings.lookup(GorgonShape::Spiral, *axis).enabled {
                &SpriteRectG::HIGHLIGHT
            } else {
                &SpriteRectG::FG
            };
            self.sprite.draw2(&m2, &sprite, gpu_state, fg)?;
        }

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
                .draw2(&m2, &freq_sprite, gpu_state, &SpriteRectG::FG)?;

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
        self.update_gorgon_val_f(val, gpu_state);

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
                gpu_state,
                &SpriteRectG::FG,
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
        settings: &MultiGorgonSettings,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitude.draw(&m2, gpu_state)?;
        }

        let latitude_enable =
            self.cursor.row == GorgonShape::Latitude && self.cursor.subrow == GorgonParam::Enable;
        if latitude_enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        for (i, (sprite, axis)) in [
            (self.sprites.x(), GorgonAxis::X),
            (self.sprites.y(), GorgonAxis::Y),
            (self.sprites.z(), GorgonAxis::Z),
        ]
        .iter()
        .enumerate()
        {
            let x = -0.25 + i as f32 * 0.5;
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y0 + sprite.scale[1], -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            let fg = if settings.lookup(GorgonShape::Latitude, *axis).enabled {
                &SpriteRectG::HIGHLIGHT
            } else {
                &SpriteRectG::FG
            };
            self.sprite.draw2(&m2, &sprite, gpu_state, fg)?;
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
                .draw2(&m2, &freq_sprite, gpu_state, &SpriteRectG::FG)?;

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
        settings: &MultiGorgonSettings,
    ) -> Result<f32, GLErrorWrapper> {
        let y = y0 + 0.25;
        {
            let m2 = matrix
                * xr_matrix4x4f_create_translation(-0.75, y, 0.0)
                * xr_matrix4x4f_uniform_scale(0.25);
            self.latitwod.draw(&m2, gpu_state)?;
        }

        if self.cursor.row == GorgonShape::Cartesian && self.cursor.subrow == GorgonParam::Enable {
            let x = self.cursor.axis.x1();
            *ring_loc = Some(SpriteLocation::new(
                [0.25; 2],
                [x, y],
                &self.sprites.texture,
            ));
        }

        for (i, (sprite, axis)) in [
            (self.sprites.x(), GorgonAxis::X),
            (self.sprites.y(), GorgonAxis::Y),
            (self.sprites.z(), GorgonAxis::Z),
        ]
        .iter()
        .enumerate()
        {
            let x = -0.25 + i as f32 * 0.5;
            let m2 = matrix
                * xr_matrix4x4f_create_translation(x, y0 + sprite.scale[1], -0.01)
                * xr_matrix4x4f_uniform_scale(0.25);
            let fg = if settings.lookup(GorgonShape::Cartesian, *axis).enabled {
                &SpriteRectG::HIGHLIGHT
            } else {
                &SpriteRectG::FG
            };
            self.sprite.draw2(&m2, &sprite, gpu_state, fg)?;
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
                .draw2(&m2, &freq_sprite, gpu_state, &SpriteRectG::FG)?;

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

    fn update_gorgon_val_f(&self, val2: FormattableValue, gpu_state: &mut GPUState) {
        let stale = match self.gorgon_val.borrow().as_ref() {
            Some(editor) => editor.val != val2,
            _ => true,
        };

        if stale {
            let font = default_font().unwrap();
            let msg = format!("{}", val2);
            if let Ok((texture, w, h)) =
                text_painting::text_to_greyscale_texture(40.0, &msg, font, gpu_state)
            {
                *(self.gorgon_val.borrow_mut()) =
                    Some(ValueEditor::new(val2, texture, w as _, h as _));
            }
        }
    }

    pub fn handle_a_click(&mut self, settings: &mut MultiGorgonSettings) {
        if let GorgonParam::Enable = self.cursor.subrow {
            settings.toggle_enabled(self.cursor)
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
