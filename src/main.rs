use ggez::conf::{WindowMode, WindowSetup};
use ggez::graphics::{self, FilterMode};
use ggez::nalgebra as na;
use ggez::{event, Context, GameResult};
use std::env;

pub mod chip8;
use chip8::Chip8;

fn main() -> GameResult {
    let window_setup = WindowSetup::default().title("chip8.rs");
    let window_mode = WindowMode::default().dimensions(64.0, 32.0);
    let cb = ggez::ContextBuilder::new("chip8.rs", "alex garrett")
        .window_setup(window_setup)
        .window_mode(window_mode);
    let (mut ctx, mut event_loop) = cb.build()?;

    let mut state = MainState::new(&mut ctx)?;

    graphics::set_drawable_size(&mut ctx, 320.0, 160.0)?;
    event::run(&mut ctx, &mut event_loop, &mut state)
}

struct MainState {
    system: Chip8,
    origin: na::Point2<f32>,
    debug: bool,
    step: bool,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let args: Vec<String> = env::args().collect();
        let mut debug = false;
        if let Some(val) = args.get(3) {
            if val == "-d" {
                debug = true;
            }
        }

        let clock_speed = match args[2].parse() {
            Ok(val) => val,
            Err(_) => 600,
        };

        let system = Chip8::new(clock_speed);
        let mut s = MainState {
            system: system,
            origin: na::Point2::new(0.0, 0.0),
            debug: debug,
            step: false,
        };

        s.system.load_rom(&args[1])?;

        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.debug {
            if self.step {
                self.system.cycle();
                self.step = false;
            }
        } else {
            self.system.cycle();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());

        let mut image = self.system.image_from_display(ctx)?;
        image.set_filter(FilterMode::Nearest);
        graphics::draw(ctx, &image, (self.origin,))?;

        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            event::KeyCode::Space => {
                if self.debug {
                    self.step = true;
                }
            }
            event::KeyCode::Key1 => {
                self.system.press_key(1);
            }
            event::KeyCode::Key2 => {
                self.system.press_key(2);
            }
            event::KeyCode::Key3 => {
                self.system.press_key(3);
            }
            event::KeyCode::Q => {
                self.system.press_key(4);
            }
            event::KeyCode::W => {
                self.system.press_key(5);
            }
            event::KeyCode::E => {
                self.system.press_key(6);
            }
            event::KeyCode::A => {
                self.system.press_key(7);
            }
            event::KeyCode::S => {
                self.system.press_key(8);
            }
            event::KeyCode::D => {
                self.system.press_key(9);
            }
            event::KeyCode::Z => {
                self.system.press_key(10);
            }
            event::KeyCode::C => {
                self.system.press_key(11);
            }
            event::KeyCode::Key4 => {
                self.system.press_key(12);
            }
            event::KeyCode::R => {
                self.system.press_key(13);
            }
            event::KeyCode::F => {
                self.system.press_key(14);
            }
            event::KeyCode::V => {
                self.system.press_key(15);
            }
            event::KeyCode::X => {
                self.system.press_key(0);
            }
            _ => {}
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
    ) {
        match keycode {
            event::KeyCode::Key1 => {
                self.system.unpress_key(1);
            }
            event::KeyCode::Key2 => {
                self.system.unpress_key(2);
            }
            event::KeyCode::Key3 => {
                self.system.unpress_key(3);
            }
            event::KeyCode::Q => {
                self.system.unpress_key(4);
            }
            event::KeyCode::W => {
                self.system.unpress_key(5);
            }
            event::KeyCode::E => {
                self.system.unpress_key(6);
            }
            event::KeyCode::A => {
                self.system.unpress_key(7);
            }
            event::KeyCode::S => {
                self.system.unpress_key(8);
            }
            event::KeyCode::D => {
                self.system.unpress_key(9);
            }
            event::KeyCode::Z => {
                self.system.unpress_key(10);
            }
            event::KeyCode::C => {
                self.system.unpress_key(11);
            }
            event::KeyCode::Key4 => {
                self.system.unpress_key(12);
            }
            event::KeyCode::R => {
                self.system.unpress_key(13);
            }
            event::KeyCode::F => {
                self.system.unpress_key(14);
            }
            event::KeyCode::V => {
                self.system.unpress_key(15);
            }
            event::KeyCode::X => {
                self.system.unpress_key(0);
            }
            _ => {}
        }
    }
}
