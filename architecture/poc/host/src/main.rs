//! The host binary: owns a window and a CPU pixel surface, loads a WASM bundle,
//! and drives it through the frozen ABI.
//!
//! This file contains NO layout logic and NO reference to any specific layout
//! library. It loads whatever `.wasm` path it is given. That obliviousness is
//! the entire point of the proof (see milestone M3).

mod engine;
mod raster;
mod text;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{Context, Result};
use engine::Bundle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const DEFAULT_W: u32 = 1280;
const DEFAULT_H: u32 = 720;

struct Args {
    bundle_path: PathBuf,
    grant_net: bool,
    dump_frame: bool,
    click: Option<(f32, f32)>,
}

fn parse_args() -> Result<Args> {
    let mut bundle_path = None;
    let mut grant_net = false;
    let mut dump_frame = false;
    let mut click = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cap" | "--cap-net" => grant_net = true,
            "--dump-frame" => dump_frame = true,
            "--click" => {
                let spec = args.next().context("--click needs an X,Y argument")?;
                let (xs, ys) = spec.split_once(',').context("--click expects X,Y")?;
                click = Some((xs.trim().parse()?, ys.trim().parse()?));
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other if other.starts_with('-') => {
                anyhow::bail!("unknown flag: {other}");
            }
            path => bundle_path = Some(PathBuf::from(path.to_string())),
        }
    }
    Ok(Args {
        bundle_path: bundle_path.context("no bundle path given\n\n(try --help)")?,
        grant_net,
        dump_frame,
        click,
    })
}

fn print_usage() {
    eprintln!("usage: host [--cap-net] [--dump-frame] <bundle.wasm>");
    eprintln!();
    eprintln!("  <bundle.wasm>   path to a WASM bundle to load and run");
    eprintln!("  --cap-net       grant the networking capability to the bundle");
    eprintln!("  --dump-frame    render one frame headlessly, print draw commands, exit");
    eprintln!("  --click X,Y     (with --dump-frame) send a click at X,Y before dumping");
}

/// The application: holds the bundle and the lazily-created window/surface.
struct App {
    bundle: Bundle,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    size: (u32, u32),
    cursor: (u16, u16),
}

impl App {
    fn new(bundle: Bundle) -> Self {
        Self {
            bundle,
            window: None,
            surface: None,
            size: (DEFAULT_W, DEFAULT_H),
            cursor: (0, 0),
        }
    }

    fn redraw(&mut self) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
        let (w, h) = self.size;
        let (Some(nw), Some(nh)) = (NonZeroU32::new(w), NonZeroU32::new(h)) else {
            return;
        };
        if surface.resize(nw, nh).is_err() {
            return;
        }

        if let Err(e) = self.bundle.render() {
            eprintln!("render error: {e:#}");
            return;
        }

        let mut buffer = match surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("surface buffer error: {e}");
                return;
            }
        };
        let (cmds, font) = self.bundle.frame();
        raster::paint(&mut buffer, w, h, cmds, font);
        if let Err(e) = buffer.present() {
            eprintln!("present error: {e}");
        }
        window.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("clean-room-web — host")
            .with_inner_size(LogicalSize::new(DEFAULT_W, DEFAULT_H));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => Rc::new(w),
            Err(e) => {
                eprintln!("failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };
        let context = match softbuffer::Context::new(window.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("failed to create softbuffer context: {e}");
                event_loop.exit();
                return;
            }
        };
        let surface = match softbuffer::Surface::new(&context, window.clone()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to create softbuffer surface: {e}");
                event_loop.exit();
                return;
            }
        };

        let phys = window.inner_size();
        self.size = (phys.width.max(1), phys.height.max(1));
        self.window = Some(window.clone());
        self.surface = Some(surface);
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.size = (size.width.max(1), size.height.max(1));
                let payload = abi::pack_u16_pair(
                    self.size.0.min(0xFFFF) as u16,
                    self.size.1.min(0xFFFF) as u16,
                );
                let _ = self.bundle.on_event(abi::event::RESIZE, payload);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as u16, position.y as u16);
                let payload = abi::pack_u16_pair(self.cursor.0, self.cursor.1);
                let _ = self.bundle.on_event(abi::event::MOUSE_MOVE, payload);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let tag = match state {
                    ElementState::Pressed => abi::event::MOUSE_DOWN,
                    ElementState::Released => abi::event::MOUSE_UP,
                };
                let payload = abi::pack_u16_pair(self.cursor.0, self.cursor.1);
                let _ = self.bundle.on_event(tag, payload);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => self.redraw(),
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let wasm = std::fs::read(&args.bundle_path)
        .with_context(|| format!("reading bundle {}", args.bundle_path.display()))?;

    let granted = if args.grant_net { abi::caps::NET } else { 0 };
    let mut bundle = Bundle::load(&wasm, granted)?;

    if args.dump_frame {
        // Populate hit regions with an initial frame, then optionally click.
        bundle.render().context("initial frame")?;
        if let Some((x, y)) = args.click {
            let payload = abi::pack_u16_pair(x as u16, y as u16);
            bundle.on_event(abi::event::MOUSE_DOWN, payload)?;
        }
        return dump_frame(&mut bundle);
    }

    let event_loop = EventLoop::new().context("creating event loop")?;
    let mut app = App::new(bundle);
    event_loop.run_app(&mut app).context("event loop")?;
    Ok(())
}

/// Render a single frame without a window and print the resulting draw commands.
///
/// This is the headless inspection path: it lets the layout output be verified
/// in CI or by eye without a display server. The host still knows nothing about
/// what the bundle is; it just prints what was pushed.
fn dump_frame(bundle: &mut Bundle) -> Result<()> {
    bundle.render().context("rendering frame")?;
    let (cmds, _font) = bundle.frame();
    println!("{} draw command(s):", cmds.len());
    for cmd in cmds {
        match cmd {
            engine::DrawCmd::Rect { x, y, w, h, rgba } => {
                println!("  rect   x={x:.0} y={y:.0} w={w:.0} h={h:.0} rgba=#{rgba:08x}");
            }
            engine::DrawCmd::Text { text, x, y, rgba } => {
                println!("  text   x={x:.0} y={y:.0} rgba=#{rgba:08x} {text:?}");
            }
        }
    }
    Ok(())
}
