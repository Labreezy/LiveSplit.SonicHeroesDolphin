#![no_std]
#![feature(type_alias_impl_trait, const_async_blocks)]
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::style,
    clippy::undocumented_unsafe_blocks,
    rust_2018_idioms
)]

use core::cmp::max;


use asr::{
    print_limited,
    emulator::gcn::{self, Emulator},
    future::{next_tick, retry},
    watcher::Watcher, timer::{self, TimerState}, time::Duration, time_util::frame_count, print_message, Address32, Address,
};
use asr::time_util;
use asr::settings::Gui;
use bitflags::bitflags;

asr::panic_handler!();
asr::async_main!(nightly);


#[derive(Gui)]
pub struct Settings {
    #[default = true]
    automatically_start: bool,
}

async fn main() {
    let mut settings = Settings::register();
    loop {
        // Hook to the target process
        let mut emulator = retry(|| gcn::Emulator::attach()).await;
        let mut watchers = Watchers::default();
        let offsets = Offsets::new();
        let mut igt_info = IGTInfo::default();


        loop {
            if !emulator.is_open() {

                break;
            }
            if emulator.update() {
                // Splitting logic. Adapted from OG LiveSplit:
                // Order of execution
                // 1. update() will always be run first. There are no conditions on the execution of this action.
                // 2. If the timer is currently either running or paused, then the isLoading, gameTime, and reset actions will be run.
                // 3. If reset does not return true, then the split action will be run.
                // 4. If the timer is currently not running (and not paused), then the start action will be run.
                update_loop(&emulator, &offsets, &mut watchers);
                let timer_state = timer::state();
                if timer_state == TimerState::Running {
                    

                    if let Some(game_time) = game_time(&watchers, &mut igt_info) {
                        
                        timer::set_game_time(game_time)
                    }
                    if split(&watchers) {
                        timer::split()
                    }
                }

                if timer::state() == TimerState::NotRunning {

                    igt_info = IGTInfo::default();
                    
                    if start(&watchers, &settings) {
                    timer::start();
                    timer::pause_game_time();
                    }
                    
                }
            }
            next_tick().await;
        }
    }
}


#[derive(Default)]
struct IGTInfo {
    total_frames: u32
}
#[derive(Default)]
struct Watchers {
    frame_counter: Watcher<u32>,
    emblem_screen: Watcher<u8>
}
struct Offsets {
    frame_counter: u32,
    emblem_screen: u32

}

impl Offsets {
    fn new() -> Self {
        Self {
            emblem_screen: 0x42C28F,
            frame_counter: 0x452C4C,
            
        }
    }
}


fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {

    let fc = game.read::<u32>(offsets.frame_counter).unwrap_or_default();
    let emblem = game.read::<u8>(offsets.emblem_screen).unwrap_or_default();    
    
    watchers.frame_counter.update_infallible(fc);
    watchers.emblem_screen.update_infallible(emblem);
    

}
    

fn start(watchers: &Watchers, settings: &Settings) -> bool {
    if !settings.automatically_start return false;
    if watchers.frame_counter.pair.expect("WHOOPS (START)").changed_from(&0) {
        return true;
    }
    return false;
}

fn split(watchers: &Watchers) -> bool {
    if watchers.emblem_screen.pair.expect("WHOOPS (SPLIT)").changed_from(&0) {
        return true;
    }
    return false;
}



fn is_loading(watchers: &Watchers) -> Option<bool> {                                                       
    Some(true)
}

fn game_time(watchers: &Watchers, info: &mut IGTInfo) -> Option<Duration> {

    let Some(fcount) = watchers.frame_counter.pair else {return None};

    let framesToAdd : u32 = fcount.current - fcount.old;
    info.total_frames += framesToAdd;
    let total_igt : Duration = frame_count::<60>(info.total_frames.into());
    Some(total_igt)
    
}