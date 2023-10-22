use android_activity::AndroidApp;
use drawcore::ActiveRenderer;
use gl_thin::gl_helper::initialize_gl_using_egli;
use jni::objects::JObject;
use jni::JavaVM;
use once_cell::sync::OnceCell;
use std::ops::Add;
use std::time::{Duration, Instant};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};
use winit::platform::android::EventLoopBuilderExtAndroid;

mod control_panel;
mod drawcore;
mod gorgon1;
mod rainbow_triangle;
mod scene;
mod shaders;
mod sprites;
mod suzanne;
mod suzanne_geometry;
mod text_painting;
mod thumbstick_smoother;
mod xr_input;

pub trait Drawable {
    fn handle_events_and_draw(&mut self);

    fn suspend(&mut self);
}

pub enum AppState<T: Drawable> {
    Paused,
    Active(T),
}

impl<T: Drawable> Default for AppState<T> {
    fn default() -> Self {
        Self::Paused
    }
}

//

fn event_loop_one_pass<T: Drawable, X: std::fmt::Debug, E: std::fmt::Debug>(
    event: Event<X>,
    event_loop: &EventLoopWindowTarget<X>,
    control_flow: &mut ControlFlow,
    app: &mut AppState<T>,
    factory: impl Fn(&EventLoopWindowTarget<X>) -> Result<T, E>,
) {
    log::trace!("Received Winit event: {event:?}");

    let static_graphics = false;

    *control_flow = match app {
        AppState::Paused => ControlFlow::Wait,
        AppState::Active(_) => {
            if static_graphics {
                ControlFlow::Poll
            } else {
                // trigger redraws every 6 milliseconds
                ControlFlow::WaitUntil(Instant::now().add(Duration::from_millis(6)))
            }
        }
    };

    match event {
        Event::Resumed => {
            log::debug!("resume");
            match factory(event_loop) {
                Ok(drawable) => *app = AppState::Active(drawable),
                Err(e) => {
                    log::error!("failed to construct drawable {:?}", e);
                }
            }
        }
        Event::Suspended => {
            log::debug!("suspend");
            if let AppState::Active(app) = app {
                app.suspend();
            }
            // log::trace!("Suspended, dropping surface state...");
            // app.surface_state = None;
            *app = AppState::Paused;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_size),
            ..
        } => {
            // Winit: doesn't currently implicitly request a redraw
            // for a resize which may be required on some platforms...
            if let AppState::Active(_) = app {
                *control_flow = ControlFlow::Poll; // this should trigger a redraw via NewEvents
            }
        }
        Event::RedrawRequested(_) => {
            log::trace!("Handling Redraw Request");
            if let AppState::Active(app) = app {
                app.handle_events_and_draw();
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::NewEvents(_) => {
            if let AppState::Active(app) = app {
                app.handle_events_and_draw();
            }
        }
        _ => {}
    }
}

static KLUDGE: OnceCell<()> = OnceCell::new();

//#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    log::debug!("bob test");

    if KLUDGE.set(()).is_err() {
        log::error!("android_main() called more than once. calling Activity.finish() to avoid EventLoop panic!");
        activity_finish(&android_app).unwrap();
    }
    let mut builder: //winit::event_loop::
        EventLoopBuilder<_> = EventLoopBuilder::new();
    let event_loop: EventLoop<()> = builder.with_android_app(android_app).build();

    log::debug!("got event loop");

    let mut app = AppState::<ActiveRenderer>::default();
    event_loop.run(move |evt, e_loop, ctx| {
        event_loop_one_pass(evt, e_loop, ctx, &mut app, |event_loop| {
            initialize_gl_using_egli();

            ActiveRenderer::new(event_loop)
        })
    });
}

fn activity_finish(android_app: &AndroidApp) -> Result<(), jni::errors::Error> {
    let vm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr() as *mut _) }?;
    let mut jvm = vm.attach_current_thread()?;
    let activity = android_app.activity_as_ptr();
    let activity = unsafe { JObject::from_raw(activity as *mut _) };
    jvm.call_method(activity, "finish", "()V", &[])?;
    log::info!(" exception check {:?}", jvm.exception_check());

    Ok(())
}
