use crate::control_panel::ControlPanel;
use crate::drawcore;
use crate::gorgon1::{Gorgon1, GorgonSettings, MultiGorgonSettings};
use crate::rainbow_triangle::RainbowTriangle;
use crate::suzanne::Suzanne;
use gl_thin::gl_fancy::GPUState;
use gl_thin::gl_helper::{explode_if_gl_error, GLErrorWrapper};
use gl_thin::linear::{
    xr_matrix4x4f_create_from_quaternion, xr_matrix4x4f_create_projection_fov,
    xr_matrix4x4f_create_scale, xr_matrix4x4f_create_translation,
    xr_matrix4x4f_create_translation_rotation_scale, xr_matrix4x4f_create_translation_v,
    xr_matrix4x4f_invert_rigid_body, xr_matrix4x4f_uniform_scale, GraphicsAPI, XrFovf,
    XrMatrix4x4f, XrQuaternionf, XrVector3f,
};
use openxr::SpaceLocation;
use openxr_sys::{Time, Vector2f};
use std::cell::RefCell;
use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MyScene {
    pub rainbow_triangle: RainbowTriangle<'static>,
    pub suzanne: Suzanne,
    pub gorgon1: RefCell<Gorgon1>,
    pub controls: ControlPanel,
    gorgon_settings: MultiGorgonSettings,
}

impl MyScene {
    pub fn new(gpu_state: &mut GPUState) -> Result<Self, GLErrorWrapper> {
        let mut gorgon_settings = MultiGorgonSettings::default();
        gorgon_settings.spirals[2] = GorgonSettings {
            enabled: true,
            frequency: 8,
            speed: 4.0,
            amplitude: 0.0,
            curl: std::f32::consts::PI,
        };
        gorgon_settings.latitudes[2] = GorgonSettings {
            enabled: true,
            frequency: 40,
            speed: 1.0,
            amplitude: 3.0,
            curl: 0.0,
        };
        Ok(MyScene {
            rainbow_triangle: RainbowTriangle::new(gpu_state)?,
            suzanne: Suzanne::new(gpu_state)?,
            gorgon1: RefCell::new(Gorgon1::new(gpu_state)?),
            controls: ControlPanel::new(gpu_state)?,
            gorgon_settings,
        })
    }

    pub fn draw(
        &self,
        fov: &XrFovf,
        rotation: &XrQuaternionf,
        translation: &XrVector3f,
        _time: Time,
        gpu_state: &mut GPUState,
        controller_1: &Option<SpaceLocation>,
    ) -> Result<(), GLErrorWrapper> {
        let (theta, rotation_matrix) = rotation_matrix_for_now();

        unsafe {
            let green = (theta.sin() + 1.0) * 0.5;
            gl::ClearColor(0.0, green, 0.3, 1.0)
        };
        explode_if_gl_error()?;
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };
        explode_if_gl_error()?;

        unsafe { gl::Enable(gl::DEPTH_TEST) };
        explode_if_gl_error()?;

        if true {
            unsafe {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }
        }

        //

        let projection_matrix =
            xr_matrix4x4f_create_projection_fov(GraphicsAPI::GraphicsOpenGL, fov, 0.01, 10_000.0);
        //log::debug!("matrix = {}", debug_string_matrix(&projection_matrix),);

        let matrix_pv = {
            let view_matrix = xr_matrix4x4f_create_translation_rotation_scale(
                translation,
                rotation,
                &XrVector3f::default_scale(),
            );
            let inverse_view_matrix = xr_matrix4x4f_invert_rigid_body(&view_matrix);

            projection_matrix * inverse_view_matrix
        };

        let skybox_pv = {
            let inverse_view_matrix = drawcore::skybox_view_matrix(rotation);
            projection_matrix * inverse_view_matrix
        };

        //

        let phase = {
            let now = SystemTime::now();
            let x = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|x| x.as_millis())
                .unwrap_or(0);

            let phase = x % 16000;
            (phase as f32) / 16000.0
        };

        self.gorgon1
            .borrow_mut()
            .paint(&skybox_pv, phase, &self.gorgon_settings, gpu_state)?;

        //

        unsafe { gl::Clear(gl::DEPTH_BUFFER_BIT) };
        explode_if_gl_error()?;

        //

        {
            let model = xr_matrix4x4f_create_translation(1.0, 0.0, -2.0);
            let model = model * rotation_matrix;
            self.rainbow_triangle
                .paint_color_triangle(&(matrix_pv * model), gpu_state)?;
        }

        if let Some(controller_1) = controller_1 {
            let model = {
                let translate =
                    xr_matrix4x4f_create_translation_v(&controller_1.pose.position.into());
                let upright = rotate_x2(-1.0, 0.0);
                let rotation_matrix =
                    xr_matrix4x4f_create_from_quaternion(&controller_1.pose.orientation.into());
                let scale1 = 0.05;
                let scale = xr_matrix4x4f_create_scale(scale1, scale1, scale1);
                translate * (rotation_matrix * (upright * scale))
            };
            self.suzanne.draw(
                &model,
                &matrix_pv,
                &[0.0, 1.0, 0.0],
                &[0.0, 0.0, 1.0],
                self.suzanne.index_count(),
                gpu_state,
            )?;

            let model = {
                let r1 = rotate_x2(0.0, 1.0);
                let s1 = xr_matrix4x4f_uniform_scale(0.1);
                let t1 = xr_matrix4x4f_create_translation(0.0, 0.0, -0.2);

                let translate =
                    xr_matrix4x4f_create_translation_v(&controller_1.pose.position.into());
                let rotation_matrix =
                    xr_matrix4x4f_create_from_quaternion(&controller_1.pose.orientation.into());
                translate * rotation_matrix * t1 * r1 * s1
            };

            self.controls
                .draw(&(matrix_pv * model), gpu_state, &self.gorgon_settings)?;
        }

        /* {
            let model = {
                let translate = xr_matrix4x4f_create_translation(0.0, -0.5, -3.0);
                let s = 0.2;
                let scale = xr_matrix4x4f_create_scale(s, s, s);
                let model = scale;
                // let model = upright * model;
                // let model = rotation_matrix * model;
                translate * model
            };
            let matrix = matrix_pv * model;
            self.text_message
                .draw(&matrix, self.text_message.index_count(), gpu_state)
        }*/

        Ok(())
    }

    pub(crate) fn handle_thumbstick(&mut self, delta: Vector2f) {
        self.controls
            .handle_thumbstick(delta, &mut self.gorgon_settings)
    }

    pub fn handle_a_click(&mut self) {
        self.controls.handle_a_click(&mut self.gorgon_settings)
    }
}

fn rotation_matrix_for_now() -> (f32, XrMatrix4x4f) {
    let theta = if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let tm = duration.as_millis();
        let phase = tm % 5000;
        TAU * phase as f32 / 5000.0
    } else {
        0.0
    };
    let rotation_matrix = if true {
        matrix_rotation_about_y(theta)
    } else {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0f32,
        ]
        .into()
    };
    (theta, rotation_matrix)
}

#[allow(dead_code)]
#[rustfmt::skip]
pub fn matrix_rotation_about_z(theta: f32) -> XrMatrix4x4f {
    rotate_z2(theta.cos(), theta.sin())
}

#[rustfmt::skip]
pub fn rotate_z2(cos: f32, sin: f32) -> XrMatrix4x4f {
    [
        cos, sin, 0.0, 0.0, //
        -sin, cos, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        0.0, 0.0, 0.0, 1.0f32,
    ]
    .into()
}

pub fn matrix_rotation_about_y(theta: f32) -> XrMatrix4x4f {
    rotate_y2(theta.cos(), theta.sin())
}

#[rustfmt::skip]
pub fn rotate_y2(cos: f32, sin: f32) -> XrMatrix4x4f {
    [
        cos, 0.0, sin, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        -sin, 0.0, cos, 0.0, //
        0.0, 0.0, 0.0, 1.0f32,
    ]
    .into()
}

#[allow(dead_code)]
pub fn matrix_rotation_about_x(theta: f32) -> XrMatrix4x4f {
    rotate_x2(theta.cos(), theta.sin())
}

#[rustfmt::skip]
pub fn rotate_x2(cos: f32, sin: f32) -> XrMatrix4x4f {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, cos, sin, 0.0,
        0.0, -sin, cos, 0.0,
        0.0, 0.0, 0.0, 1.0f32,
    ]
    .into()
}
