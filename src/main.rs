extern crate clap;
extern crate core_foundation;
extern crate core_graphics;
extern crate libc;
extern crate regex;

use regex::Regex;

use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::{
    kCGDisplayShowDuplicateLowResolutionModes, CGConfigureOption, CGDisplay, CGDisplayMode,
};

use clap::{App, Arg, SubCommand};

struct Mode {
    display: u64,
    width: u64,
    height: u64,
    pixel_width: u64,
    pixel_height: u64,
    refresh_rate: f64,
    io_flags: u32,
    bit_depth: usize,
}

fn get_current_mode() {
    println!(
        "Active display count: {}",
        CGDisplay::active_display_count().unwrap()
    );
    let displays = CGDisplay::active_displays().unwrap();
    displays
        .into_iter()
        .enumerate()
        .for_each(|(i, display_id)| {
            let display = CGDisplay::new(display_id);
            let cgmode = display.display_mode().unwrap();
            println!(
                "Display {}: {}x{}, pixel {}x{}, refresh rate: {}, flags: {}, bitDepth: {}",
                i,
                cgmode.width(),
                cgmode.height(),
                cgmode.pixel_width(),
                cgmode.pixel_height(),
                cgmode.refresh_rate(),
                cgmode.io_flags(),
                cgmode.bit_depth()
            );
        });
}

fn set_current_mode(mode: &str, display: u64) {
    println!("Setting mode: {}, display: {}", mode, display);

    // Parse: in the style of 1920x1200x32@0
    let re = Regex::new(r"(\d+)x(\d+)x(\d+)@(\d+)").unwrap();
    let captures = re.captures(mode);
    let wanted_mode = match captures {
        Some(caps) => {
            println!(
                "Display: {}: width: {}, height: {}, bitdepth: {}, refresh: {}",
                display,
                caps.get(1).unwrap().as_str(),
                caps.get(2).unwrap().as_str(),
                caps.get(3).unwrap().as_str(),
                caps.get(4).unwrap().as_str()
            );
            Some(Mode {
                display: display,
                width: caps.get(1).unwrap().as_str().parse().unwrap(),
                height: caps.get(2).unwrap().as_str().parse().unwrap(),
                pixel_width: 0,
                pixel_height: 0,
                refresh_rate: caps.get(4).unwrap().as_str().parse().unwrap(),
                io_flags: 0,
                bit_depth: caps.get(3).unwrap().as_str().parse().unwrap(),
            })
        }
        None => {
            println!("No match");
            None
        }
    };
    if let Some(wanted_mode) = wanted_mode {
        let value = CFNumber::from(1);
        let key =
            unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

        let display_ids = CGDisplay::active_displays().unwrap();
        let target_display_id = display_ids
            .into_iter()
            .enumerate()
            .filter(|(i, _)| *i as u64 == display)
            .map(|(_, display_id)| display_id)
            .next();

        if let Some(display_id) = target_display_id {
            let array_opt: Option<Vec<CGDisplayMode>> =
                CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef());
            let modes = array_opt.unwrap();
            let possible_index = modes
                .clone()
                .into_iter()
                .enumerate()
                .filter(|(_, cgmode)| {
                    cgmode.width() == wanted_mode.width
                        && cgmode.height() == wanted_mode.height
                        && cgmode.bit_depth() == wanted_mode.bit_depth
                        && cgmode.refresh_rate() == wanted_mode.refresh_rate
                }).map(|(i, _)| i)
                .next();

            if let Some(index) = possible_index {
                let display = CGDisplay::new(display_id);
                let config_ref = display.begin_configuration();
                if let Ok(config_ref) = config_ref {
                    let result =
                        display.configure_display_with_display_mode(&config_ref, &modes[index]);
                    match result {
                        Ok(()) => {
                            let result = display.complete_configuration(
                                &config_ref,
                                CGConfigureOption::ConfigurePermanently,
                            );
                            match result {
                                Ok(()) => {
                                    println!("Settings applied!");
                                }
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
            }
        }
    }
}

fn obtain_all_modes_for_all_displays() -> Vec<Mode> {
    let mut result: Vec<Mode> = Vec::with_capacity(50);
    let value = CFNumber::from(1);
    let key = unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    let display_ids = CGDisplay::active_displays().unwrap();

    display_ids
        .into_iter()
        .enumerate()
        .for_each(|(i, display_id)| {
            let array_opt: Option<Vec<CGDisplayMode>> =
                CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef());
            let modes = array_opt.unwrap();

            modes.into_iter().for_each(|cgmode| {
                result.push(Mode {
                    display: i as u64,
                    width: cgmode.width(),
                    height: cgmode.height(),
                    pixel_width: cgmode.pixel_width(),
                    pixel_height: cgmode.pixel_height(),
                    refresh_rate: cgmode.refresh_rate(),
                    io_flags: cgmode.io_flags(),
                    bit_depth: cgmode.bit_depth(),
                });
            });
        });
    result
}

fn list_modes() {
    obtain_all_modes_for_all_displays()
        .into_iter()
        .for_each(|cgmode| {
            println!(
                "Display {}: {}x{}, pixel {}x{}, refresh rate: {}, flags: {}, bitDepth: {}",
                cgmode.display,
                cgmode.width,
                cgmode.height,
                cgmode.pixel_width,
                cgmode.pixel_height,
                cgmode.refresh_rate,
                cgmode.io_flags,
                cgmode.bit_depth
            );
        });
}

fn main() {
    let matches = App::new("MacOS Screen Resolution Tool")
        .version("0.1.0")
        .author("Bernard Niset")
        .about("Allows to list, get and set screen resolutions.")
        .subcommand(
            SubCommand::with_name("list").about("List available resolutions for current display"),
        ).subcommand(
            SubCommand::with_name("get").about("Get current active resution for current display"),
        ).subcommand(
            SubCommand::with_name("set")
                .about("Set current active resolution for current display (TODO)")
                .arg(
                    Arg::with_name("display")
                        .long("display")
                        .value_name("DISPLAY")
                        .short("d")
                        .takes_value(true),
                ).arg(
                    Arg::with_name("resolution")
                        .value_name("RESOLUTION")
                        .help("Resolution string in the form of 1920x1200x32@0")
                        .required(true)
                        .takes_value(true),
                ),
        ).get_matches();
    match matches.subcommand() {
        ("list", Some(_sub_m)) => {
            list_modes();
        }
        ("get", Some(_sub_m)) => get_current_mode(),
        ("set", Some(sub_m)) => {
            let display = sub_m
                .value_of("display")
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);

            set_current_mode(sub_m.value_of("resolution").unwrap(), display);
        }
        _ => {}
    }
}
