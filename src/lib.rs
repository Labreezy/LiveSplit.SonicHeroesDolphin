#![no_std]

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


asr::panic_handler!();
asr::async_main!(stable);


#[derive(Gui)]
pub struct Settings {
    #[default = true]
    ///Start
    ///
    ///Automatically start the autosplitter.
    automatically_start: bool,
}
//G9SE8P
const GAME_ID_US: [u8;6] = [0x47, 0x39, 0x53, 0x45, 0x38, 0x50];
//G9SJ8P
const GAME_ID_JP: [u8;6] = [0x47, 0x39, 0x53, 0x4A, 0x38, 0x50];
async fn main() {
    let mut settings = Settings::register();
    loop {
        // Hook to the target process
        let mut emulator = retry(|| gcn::Emulator::attach()).await;
        let mut watchers = Watchers::default();
        let mut offsets = Offsets::default();
        let mut igt_info = IGTInfo::default();
        

        loop {
            if !emulator.is_open() {
                if offsets.emblem_screen != 0{
                    offsets = Offsets::default();
                }
                break;
            }
            if emulator.update() {
                // Splitting logic. Adapted from OG LiveSplit:
                // Order of execution
                // 1. update() will always be run first. There are no conditions on the execution of this action.
                // 2. If the timer is currently either running or paused, then the isLoading, gameTime, and reset actions will be run.
                // 3. If reset does not return true, then the split action will be run.
                // 4. If the timer is currently not running (and not paused), then the start action will be run.
                settings.update();
                if offsets.emblem_screen == 0{
                    let game_id = emulator.read::<[u8; 6]>(0x80000000).unwrap_or_default();
                    for b in game_id {
                        print_limited::<64>(&format_args!("{b:#02X}"));
                    }
                    print_message("-------");
                    if game_id[3] == 0x4A {
                        print_message("JP HEROES");
                    } else if game_id[3] == 0x45 {
                        print_message("US HEROES");
                    }
                    
                    offsets = Offsets::from_gameid(game_id);
                    
                }
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
            } else if offsets.emblem_screen != 0{
                offsets = Offsets::default();
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


#[derive(Default)]
struct Offsets {
    frame_counter: u32,
    emblem_screen: u32

}


impl Offsets {
    fn from_gameid(game_id: [u8;6]) -> Offsets {
        
        let offsets = match game_id {
            GAME_ID_US => Self {
                emblem_screen: 0x42C28F,
                frame_counter: 0x452C4C,
                
            }
        ,
            GAME_ID_JP => Self {
                emblem_screen: 0x42C36F,
                frame_counter: 0x442D4C,
                
            },
            _ => Self::default()
        };
        return offsets;
        }
       
    }





fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {
   
    if offsets.emblem_screen == 0 {
        return
    }
    let fc = game.read::<u32>(offsets.frame_counter).unwrap_or_default();
    let emblem = game.read::<u8>(offsets.emblem_screen).unwrap_or_default();    
    
    watchers.frame_counter.update_infallible(fc);
    watchers.emblem_screen.update_infallible(emblem);
    

}
    

fn start(watchers: &Watchers, settings: &Settings) -> bool {
    if !settings.automatically_start {return false;}
    if watchers.frame_counter.pair.unwrap_or_default().changed_from(&0) {
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
    if(fcount.current > fcount.old){
        let framesToAdd : u32 = fcount.current - fcount.old;
        info.total_frames += framesToAdd;
    }
    
    let total_igt : Duration = frame_count::<60>(info.total_frames.into());
    Some(total_igt)
    
}