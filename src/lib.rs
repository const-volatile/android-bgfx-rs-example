#![allow(dead_code)]
use bgfx_rs::bgfx;
use bgfx_rs::bgfx::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::window::Window;
#[cfg(target_os = "android")]
use ndk::native_window::NativeWindow;

#[cfg(target_os = "linux")]
fn get_render_type() -> RendererType { RendererType::OpenGL }
#[cfg(target_os = "android")]
fn get_render_type() -> RendererType { RendererType::OpenGLES }
#[cfg(all(not(target_os = "linux"), not(target_os = "android")))]
fn get_render_type() -> RendererType { RendererType::Count }

fn update_platform_handle(pd: &mut PlatformData, window: &Window) {
    match window.raw_window_handle() {
        #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
        ))]
        RawWindowHandle::Xlib(data) => {
            pd.nwh = data.window as *mut _;
            pd.ndt = data.display as *mut _;
        }
        #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
        ))]
        RawWindowHandle::Wayland(data) => {
            pd.ndt = data.surface; // same as window, on wayland there ins't a concept of windows
            pd.nwh = data.display;
        }
        #[cfg(target_os = "macos")]
        RawWindowHandle::MacOS(data) => {
            pd.nwh = data.ns_window;
        }
        #[cfg(target_os = "windows")]
        RawWindowHandle::Windows(data) => {
            pd.nwh = data.hwnd;
        }
        #[cfg(target_os = "android")]
        RawWindowHandle::Android(data) => {
            pd.nwh = data.a_native_window;
        }
        _ => panic!("Unsupported Window Manager"),
    }
}

#[cfg(not(target_os = "android"))]
fn init_bgfx(window: &Window){
    let mut pd = bgfx::PlatformData::new();
    update_platform_handle(&mut pd, &window);
    bgfx::set_platform_data(&pd);
    let mut init = Init::new();
    init.type_r = get_render_type();
    init.resolution.width = window.inner_size().width as u32;
    init.resolution.height = window.inner_size().height as u32;
    init.resolution.reset = ResetFlags::VSYNC.bits();
    init.platform_data = pd;
    if !bgfx::init(&init) {
        panic!("failed to init bgfx");
    }
    bgfx::set_view_rect(0, 0, 0, window.inner_size().width as u16, window.inner_size().height as u16);
}

#[cfg(target_os = "android")]
fn init_bgfx(window: &NativeWindow){
    let mut pd = bgfx::PlatformData::new();
    pd.nwh = window.ptr().as_ptr() as *mut std::ffi::c_void;
    let mut init = Init::new();
    init.type_r = bgfx_rs::bgfx::RendererType::OpenGLES;
    init.resolution.width = window.width() as u32;
    init.resolution.height = window.height() as u32;
    init.resolution.reset = ResetFlags::VSYNC.bits();
    init.platform_data = pd;
    if !bgfx::init(&init) {
        panic!("failed to init bgfx");
    }
}

#[cfg(target_os = "android")]
#[ndk_glue::main(backtrace = "on")]
pub fn main() {
    start();
}

pub fn start() {
    let el = EventLoop::new();
    #[cfg(not(target_os = "android"))]
    let window = winit::window::WindowBuilder::new()
        .with_title("App!")
        .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
        .build(&el)
        .unwrap();
    #[cfg(not(target_os = "android"))]
    init_bgfx(&window);
    let mut initalized = false;
    el.run(move |event, _, control_flow| {
        println!("{:?}", event);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {
                return
            },
            Event::Suspended => {
                if initalized {
                    initalized = false;
                    bgfx::shutdown();
                }
            },
            Event::Resumed => {
                #[cfg(target_os = "android")]
                {
                    let window = ndk_glue::native_window();
                    match window.as_ref() {
                        Some(window) => {
                            init_bgfx(window);
                            initalized = true;
                        },
                        None => { }
                    };
                }
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    if initalized {
                        bgfx::reset(physical_size.width, physical_size.height, ResetArgs::default());
                    }
                },
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            //Event::MainEventsCleared => {
            //    window.request_redraw();
            //},
            Event::RedrawRequested(_) => {
                bgfx::set_debug(DebugFlags::TEXT.bits());
                bgfx::set_view_clear(
                    0,
                    ClearFlags::COLOR.bits() | ClearFlags::DEPTH.bits(),
                    SetViewClearArgs {
                        rgba: 0x103030ff,
                        ..Default::default()
                    },
                );
                #[cfg(target_os = "android")]
                {
                    let window = ndk_glue::native_window();
                    match window.as_ref() {
                        Some(window) => {
                            bgfx::set_view_rect(0, 0, 0, window.width() as _, window.height() as _);
                        },
                        None => {}
                    };
                }

                bgfx::touch(0);
                bgfx::dbg_text_clear(DbgTextClearArgs::default());
                bgfx::dbg_text(0, 1, 0x0f, "Color can be changed with ANSI \x1b[9;me\x1b[10;ms\x1b[11;mc\x1b[12;ma\x1b[13;mp\x1b[14;me\x1b[0m code too.");
                bgfx::dbg_text(80, 1, 0x0f, "\x1b[;0m    \x1b[;1m    \x1b[; 2m    \x1b[; 3m    \x1b[; 4m    \x1b[; 5m    \x1b[; 6m    \x1b[; 7m    \x1b[0m");
                bgfx::dbg_text(80, 2, 0x0f, "\x1b[;8m    \x1b[;9m    \x1b[;10m    \x1b[;11m    \x1b[;12m    \x1b[;13m    \x1b[;14m    \x1b[;15m    \x1b[0m");
                bgfx::dbg_text(
                    0,
                    4,
                    0x3f,
                    "Description: Initialization and debug text with bgfx-rs Rust API.",
                );
                bgfx::frame(false);
            }
            _ => {
                //println!("Unhandled event: {:?}", event);
            },
        }
    });
}