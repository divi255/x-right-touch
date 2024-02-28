use clap::Parser;
use inputbot::MouseButton::LeftButton;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType};
use std::{
    mem,
    sync::atomic,
    thread,
    time::{Duration, Instant},
};

#[derive(Parser)]
struct Args {
    #[clap(long, default_value = "500", help = "touch wait time, in milliseconds")]
    wait: u16,
    #[clap(short = 'v', long, default_value = "false")]
    verbose: bool,
}

#[derive(Default)]
struct State {
    mouse_coords: Option<(f64, f64)>,
    mouse_moved_delta: f64,
    mouse_press: Option<Instant>,
    maybe_screen_press: Option<Instant>,
    need_simulate: bool,
}

static STATE: Lazy<Mutex<State>> = Lazy::new(<_>::default);
static WAIT: atomic::AtomicU16 = atomic::AtomicU16::new(0);
static HANDLER_ACTIVE: atomic::AtomicBool = atomic::AtomicBool::new(false);

const SLEEP_STEP: Duration = Duration::from_millis(50);
const PRESS_DELAY: Duration = Duration::from_millis(10);

fn callback(event: Event) {
    match event.event_type {
        EventType::MouseMove { x, y } => {
            let mut state = STATE.lock();
            if state.maybe_screen_press.is_some() {
                if let Some(coords) = state.mouse_coords {
                    let delta = (coords.0 - x).abs() + (coords.1 - y).abs();
                    if state.mouse_moved_delta < delta {
                        state.mouse_moved_delta = delta;
                    }
                } else {
                    state.mouse_coords = Some((x, y));
                }
            }
        }
        EventType::ButtonPress(rdev::Button::Left) => {
            debug!("maybe screen touch");
            spawn_handler();
        }
        EventType::ButtonRelease(rdev::Button::Left) => {
            debug!("maybe screen release");
            let need_simulate = {
                let mut state = STATE.lock();
                state.maybe_screen_press = None;
                if state.mouse_moved_delta > 40.0 {
                    debug!("drag event ({})", state.mouse_moved_delta);
                    false
                } else {
                    mem::take(&mut state.need_simulate)
                }
            };
            if need_simulate {
                if let Err(e) = simulate_right_click() {
                    error!("unable to simulate right click: {}", e);
                }
            }
        }
        _ => {}
    }
}

fn configure_env_logger(verbose: bool) {
    let mut builder = env_logger::Builder::new();
    builder.target(env_logger::Target::Stdout);
    builder.filter_level(if verbose {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    });
    builder.init();
}

fn spawn_handler() {
    if HANDLER_ACTIVE.load(atomic::Ordering::Relaxed) {
        debug!("gesture handler already active, aborting");
        return;
    }
    HANDLER_ACTIVE.store(true, atomic::Ordering::Relaxed);
    let mut state = STATE.lock();
    state.maybe_screen_press = Some(Instant::now());
    state.mouse_moved_delta = 0.0;
    state.mouse_coords = None;
    let touch_wait = Duration::from_millis(WAIT.load(atomic::Ordering::Relaxed).into());
    thread::spawn(move || {
        debug!("gesture handler started");
        loop {
            thread::sleep(SLEEP_STEP);
            let mut state = STATE.lock();
            if let Some(maybe_screen) = state.maybe_screen_press {
                if let Some(mouse) = state.mouse_press {
                    if mouse.elapsed() < touch_wait {
                        state.maybe_screen_press = None;
                        state.mouse_moved_delta = 0.0;
                        break;
                    }
                }
                if maybe_screen.elapsed() < touch_wait {
                    continue;
                }
                state.maybe_screen_press = None;
                state.mouse_press = None;
                state.need_simulate = true;
                break;
            }
        }
        HANDLER_ACTIVE.store(false, atomic::Ordering::Relaxed);
        debug!("gesture handler exited");
    });
}

fn simulate_right_click() -> Result<(), Box<dyn std::error::Error>> {
    info!("simulating");
    rdev::simulate(&EventType::ButtonPress(rdev::Button::Right))?;
    thread::sleep(PRESS_DELAY);
    rdev::simulate(&EventType::ButtonRelease(rdev::Button::Right))?;
    Ok(())
}

fn main() {
    let args = Args::parse();
    WAIT.store(args.wait, atomic::Ordering::Relaxed);
    configure_env_logger(args.verbose);
    thread::spawn(move || {
        LeftButton.bind(|| {
            debug!("left mouse button press");
            STATE.lock().mouse_press = Some(Instant::now());
        });
        inputbot::handle_input_events();
    });
    if let Err(error) = listen(callback) {
        error!("{:?}", error);
        std::process::exit(1);
    }
}
