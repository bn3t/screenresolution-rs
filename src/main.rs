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

type DisplayId = u32;
type DisplayIndex = u8;

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

enum ScreenFormat {
    F16_9,
    F16_10,
    F4_3,
}
struct Mode {
    display: DisplayIndex,
    width: u64,
    height: u64,
    pixel_width: u64,
    pixel_height: u64,
    refresh_rate: f64,
    io_flags: u32,
    bit_depth: usize,
}

impl Mode {
    fn from(display: DisplayIndex, cgmode: &CGDisplayMode) -> Mode {
        Mode {
            display: display,
            width: cgmode.width(),
            height: cgmode.height(),
            pixel_width: cgmode.pixel_width(),
            pixel_height: cgmode.pixel_height(),
            refresh_rate: cgmode.refresh_rate(),
            io_flags: cgmode.io_flags(),
            bit_depth: cgmode.bit_depth(),
        }
    }

    fn is_hdpi(&self) -> bool {
        self.width != self.pixel_width || self.height != self.pixel_height
    }

    fn screen_format(&self) -> ScreenFormat {
        let f16_9 = 16_f64 / 9_f64;
        let f16_10 = 16_f64 / 10_f64;
        let screen_format = self.width as f64 / self.height as f64;
        if screen_format == f16_9 {
            ScreenFormat::F16_9
        } else if screen_format == f16_10 {
            ScreenFormat::F16_10
        } else {
            ScreenFormat::F4_3
        }
    }

    fn print_short(&self) {
        let hidpi = if self.is_hdpi() { "HiDPI" } else { "" };
        let screen_format = match self.screen_format() {
            ScreenFormat::F16_9 => "16:9",
            ScreenFormat::F16_10 => "16:10",
            ScreenFormat::F4_3 => "4:3",
        };

        let mode_str = format!(
            "{}x{}x{}@{}",
            self.width, self.height, self.bit_depth, self.refresh_rate
        );
        let mode_pixel = format!(
            "{}x{}x{}@{}",
            self.pixel_width, self.pixel_height, self.bit_depth, self.refresh_rate
        );
        println!(
            "Display {}: {:15} - pixel {:15} - {:6} - {:6}",
            self.display, mode_str, mode_pixel, hidpi, screen_format
        );
    }

    fn print_long(&self) {
        let hidpi = if self.is_hdpi() { "HiDPI" } else { "" };
        let screen_format = match self.screen_format() {
            ScreenFormat::F16_9 => "16:9",
            ScreenFormat::F16_10 => "16:10",
            ScreenFormat::F4_3 => "4:3",
        };
        println!(
            "Display {}: {}x{}, refresh rate: {}, bitDepth: {}, flags: 0x{:07X}, {}, {}",
            self.display,
            self.width,
            self.height,
            self.refresh_rate,
            self.bit_depth,
            self.io_flags,
            hidpi,
            screen_format
        );
    }

    fn print_mode(&self, short: bool) {
        match short {
            true => self.print_short(),
            false => self.print_long(),
        }
    }
}
impl PartialEq for Mode {
    fn eq(&self, other: &Mode) -> bool {
        self.display == other.display && self.width == other.width && self.height == self.height
    }
}

fn get_current_mode_for_display(
    display_index: DisplayIndex,
    display_id: DisplayId,
) -> Option<Mode> {
    let display = CGDisplay::new(display_id);
    display
        .display_mode()
        .map(|cgmode| Mode::from(display_index, &cgmode))
}

fn print_current_mode(short: bool) -> Result<()> {
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
            let mode = get_current_mode_for_display(i as DisplayIndex, display_id).unwrap();
            mode.print_mode(short);
        });
    Ok(())
}

fn parse_wanted_mode(mode: &str, display: DisplayIndex) -> Result<Option<Mode>> {
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

fn configure_display(cgmode: &CGDisplayMode, display_id: DisplayId) -> Result<()> {
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

// Return true if the current mode is different from specified mode
fn verify_current(mode: &Mode, display_index: DisplayIndex, display_id: DisplayId) -> bool {
    get_current_mode_for_display(display_index, display_id)
        .map(|current_mode| current_mode != *mode)
        .unwrap_or(false)
}

fn get_display_id_from_display_index(display_index: DisplayIndex) -> Option<DisplayId> {
    let display_ids = CGDisplay::active_displays().unwrap();
    display_ids
        .into_iter()
        .enumerate()
        .filter(|(i, _)| *i as DisplayIndex == display_index)
        .map(|(_, display_id)| display_id)
        .next()
}

fn set_current_mode(mode: &str, display_index: DisplayIndex) -> Result<()> {
    // println!("Setting mode: {}, display: {}", mode, display);
    let wanted_mode =
        parse_wanted_mode(mode, display_index).chain_err(|| "Could not parse wanted mode")?;
    if let Some(wanted_mode) = wanted_mode {
        let target_display_id = get_display_id_from_display_index(display_index);
        if let Some(display_id) = target_display_id {
            if verify_current(&wanted_mode, display_index, display_id) {
                let cgmodes = all_display_modes(display_id).unwrap();
                let possible_index = cgmodes
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
                    configure_display(&cgmodes[index], display_id)
                        .chain_err(|| "Could not actually configure display")?;
                }
                Ok(())
            } else {
                Err("Wanted Mode is already current".into())
            }
        } else {
            Err(format!("Not a valid display: {}", display_index).into())
        }
    } else {
        Err(format!("Not a valid mode: {}", mode).into())
    }
}

fn all_display_modes(display_id: DisplayId) -> Option<Vec<CGDisplayMode>> {
    let value = CFNumber::from(1);
    let key = unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef())
}

fn obtain_all_modes_for_all_displays() -> Result<Vec<Mode>> {
    let mut result: Vec<Mode> = Vec::with_capacity(50);

    let display_ids = convert_result(CGDisplay::active_displays())
        .chain_err(|| "Unable to list active displays")?;

    display_ids
        .into_iter()
        .enumerate()
        .for_each(|(i, display_id)| {
            let array_opt: Option<Vec<CGDisplayMode>> = all_display_modes(display_id);
            let modes = array_opt.unwrap();

            modes.into_iter().for_each(|cgmode| {
                let io_flags = cgmode.io_flags();
                if (io_flags & (kDisplayModeValidFlag | kDisplayModeSafeFlag)) != 0 {
                    result.push(Mode::from(i as DisplayIndex, &cgmode));
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
    all_modes.into_iter().for_each(|mode| {
        mode.print_mode(short);
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
            print_current_mode(short)
        }
        ("set", Some(sub_m)) => {
            let display = sub_m
                .value_of("display")
                .unwrap_or("0")
                .parse::<DisplayIndex>()
                .unwrap_or(0);
            set_current_mode(sub_m.value_of("resolution").unwrap(), display)
        }
        _ => Ok(()),
    }
}

quick_main!(run);
