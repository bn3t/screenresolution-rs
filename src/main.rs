#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

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
    kCGDisplayShowDuplicateLowResolutionModes, kDisplayModeSafeFlag, kDisplayModeValidFlag,
    CGConfigureOption, CGDisplay, CGDisplayMode,
};

use clap::{App, AppSettings, Arg, SubCommand};

mod errors {
    use core_graphics::base;
    use std::error;
    use std::fmt;
    use std::result;

    #[derive(Debug)]
    pub struct CGError {
        error: base::CGError,
    }

    impl error::Error for CGError {
        fn description(&self) -> &str {
            "a CG error"
        }

        fn cause(&self) -> Option<&error::Error> {
            None
        }
    }

    impl fmt::Display for CGError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "CGError: {}", self)
        }
    }

    impl From<base::CGError> for CGError {
        fn from(e: base::CGError) -> Self {
            CGError { error: e }
        }
    }

    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        foreign_links {
            CgError(CGError);
        }
    }

    pub fn convert_result<T>(
        result: result::Result<T, base::CGError>,
    ) -> result::Result<T, CGError> {
        result.map_err(|e: base::CGError| CGError { error: e })
    }
}

use errors::*;

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

fn print_mode(
    short: bool,
    display: u32,
    width: u64,
    height: u64,
    pixel_width: u64,
    pixel_height: u64,
    refresh_rate: f64,
    bit_depth: usize,
    io_flags: u32,
) {
    let hidpi = match width != pixel_width || height != pixel_height {
        true => "HiDPI",
        false => "",
    };
    let f16_9 = 16_f64 / 9_f64;
    let f16_10 = 16_f64 / 10_f64;
    let screen_format = width as f64 / height as f64;
    let screen_format = if screen_format == f16_9 {
        "16:9"
    } else if screen_format == f16_10 {
        "16:10"
    } else {
        "4:3"
    };

    if short {
        let mode = format!("{}x{}x{}@{}", width, height, bit_depth, refresh_rate);
        let mode_pixel = format!(
            "{}x{}x{}@{}",
            pixel_width, pixel_height, bit_depth, refresh_rate
        );
        println!(
            "Display {}: {:15} - pixel {:15} - {:6} - {:6}",
            display, mode, mode_pixel, hidpi, screen_format
        );
    } else {
        println!(
            "Display {}: {}x{}, refresh rate: {}, bitDepth: {}, flags: 0x{:07X}, {}, {}",
            display, width, height, refresh_rate, bit_depth, io_flags, hidpi, screen_format
        );
    }
}

fn get_current_mode(short: bool) -> Result<()> {
    println!(
        "Active display count: {}",
        convert_result(CGDisplay::active_display_count())
            .chain_err(|| "Could not get active display count")?
    );
    let displays = convert_result(CGDisplay::active_displays())
        .chain_err(|| "Could not list active displays")?;
    displays
        .into_iter()
        .enumerate()
        .for_each(|(i, display_id)| {
            let display = CGDisplay::new(display_id);
            let cgmode = display.display_mode().unwrap();
            print_mode(
                short,
                i as u32,
                cgmode.width(),
                cgmode.height(),
                cgmode.pixel_width(),
                cgmode.pixel_height(),
                cgmode.refresh_rate(),
                cgmode.bit_depth(),
                cgmode.io_flags(),
            );
        });
    Ok(())
}

fn parse_wanted_mode(mode: &str, display: u64) -> Result<Option<Mode>> {
    // Parse: in the style of 1920x1200x32@0
    let re = Regex::new(r"(\d+)x(\d+)x(\d+)@(\d+)").chain_err(|| "Could not compile regex")?;
    let captures = re.captures(mode);
    match captures {
        Some(caps) => {
            // println!(
            //     "Display: {}: width: {}, height: {}, bitdepth: {}, refresh: {}",
            //     display,
            //     caps.get(1).unwrap().as_str(),
            //     caps.get(2).unwrap().as_str(),
            //     caps.get(3).unwrap().as_str(),
            //     caps.get(4).unwrap().as_str()
            // );
            Ok(Some(Mode {
                display: display,
                width: caps.get(1).unwrap().as_str().parse().unwrap(),
                height: caps.get(2).unwrap().as_str().parse().unwrap(),
                pixel_width: 0,
                pixel_height: 0,
                refresh_rate: caps.get(4).unwrap().as_str().parse().unwrap(),
                io_flags: 0,
                bit_depth: caps.get(3).unwrap().as_str().parse().unwrap(),
            }))
        }
        None => Ok(None),
    }
}

fn configure_display(cgmode: &CGDisplayMode, display_id: u32) -> Result<()> {
    let display = CGDisplay::new(display_id);
    let config_ref = convert_result(display.begin_configuration())
        .chain_err(|| "Could not begin configuring the display")?;
    let result = display.configure_display_with_display_mode(&config_ref, cgmode);
    match result {
        Ok(()) => {
            let result = display
                .complete_configuration(&config_ref, CGConfigureOption::ConfigurePermanently);
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
    };
    Ok(())
}

fn set_current_mode(mode: &str, display: u64) -> Result<()> {
    // println!("Setting mode: {}, display: {}", mode, display);
    let wanted_mode =
        parse_wanted_mode(mode, display).chain_err(|| "Could not parse wanted mode")?;
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
                configure_display(&modes[index], display_id)
                    .chain_err(|| "Could not actually configure display")?;
            }
            Ok(())
        } else {
            Err(format!("Not a valid display: {}", display).into())
        }
    } else {
        Err(format!("Not a valid mode: {}", mode).into())
    }
}

fn obtain_all_modes_for_all_displays() -> Result<Vec<Mode>> {
    let mut result: Vec<Mode> = Vec::with_capacity(50);
    let value = CFNumber::from(1);
    let key = unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    let display_ids = convert_result(CGDisplay::active_displays())
        .chain_err(|| "Unable to list active displays")?;

    display_ids
        .into_iter()
        .enumerate()
        .for_each(|(i, display_id)| {
            let array_opt: Option<Vec<CGDisplayMode>> =
                CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef());
            let modes = array_opt.unwrap();

            modes.into_iter().for_each(|cgmode| {
                let io_flags = cgmode.io_flags();
                if (io_flags & (kDisplayModeValidFlag | kDisplayModeSafeFlag)) != 0 {
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
                }
            });
        });
    result.sort_unstable_by(|a, b| {
        a.display
            .cmp(&(b.display))
            .then(a.width.cmp(&(b.width)).reverse())
            .then(a.height.cmp(&(b.height)).reverse())
    });
    Ok(result)
}

fn list_modes(short: bool) -> Result<()> {
    let all_modes = obtain_all_modes_for_all_displays()?;
    all_modes.into_iter().for_each(|cgmode| {
        print_mode(
            short,
            cgmode.display as u32,
            cgmode.width,
            cgmode.height,
            cgmode.pixel_width,
            cgmode.pixel_height,
            cgmode.refresh_rate,
            cgmode.bit_depth,
            cgmode.io_flags,
        );
    });
    Ok(())
}

fn run() -> Result<()> {
    let matches = App::new("MacOS Screen Resolution Tool")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Bernard Niset")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .after_help(format!("Build: {} - {}", env!("GIT_COMMIT"), env!("BUILD_DATE")).as_str())
        .subcommand(
            SubCommand::with_name("list")
                .about("List available resolutions for current display")
                .arg(
                    Arg::with_name("short")
                        .long("short")
                        .short("s")
                        .required(false)
                        .takes_value(false),
                ),
        ).subcommand(
            SubCommand::with_name("get")
                .about("Get current active resution for current display")
                .arg(
                    Arg::with_name("short")
                        .long("short")
                        .short("s")
                        .required(false)
                        .takes_value(false),
                ),
        ).subcommand(
            SubCommand::with_name("set")
                .about("Set current active resolution for current display")
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
        ("list", Some(sub_m)) => {
            let short = sub_m.is_present("short");
            list_modes(short)
        }
        ("get", Some(sub_m)) => {
            let short = sub_m.is_present("short");
            get_current_mode(short)
        }
        ("set", Some(sub_m)) => {
            let display = sub_m
                .value_of("display")
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            set_current_mode(sub_m.value_of("resolution").unwrap(), display)
        }
        _ => Ok(()),
    }
}

quick_main!(run);
