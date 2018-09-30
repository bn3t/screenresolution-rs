#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate clap;
extern crate core_foundation;
extern crate core_graphics;
extern crate dialoguer;
extern crate libc;
extern crate regex;

use regex::Regex;
use std::io;

use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;

use core_graphics::display::{
    kCGDisplayShowDuplicateLowResolutionModes, kDisplayModeSafeFlag, kDisplayModeValidFlag,
    CGConfigureOption, CGDirectDisplayID, CGDisplay, CGDisplayMode,
};

use clap::{App, AppSettings, Arg, ArgGroup, SubCommand};

use dialoguer::Select;

mod errors;
mod mode;

use errors::*;
use mode::*;

/// ScreenResolution struct to hold the app main state:
/// * A vec of displays
/// * A vec of Modes corresponding to all Modes available for all displays.
struct ScreenResolution {
    displays: Vec<CGDirectDisplayID>,
    modes: Vec<Mode>,
}

impl ScreenResolution {
    pub fn new() -> Result<Self> {
        let mut modes: Vec<Mode> = Vec::with_capacity(50);

        let displays = convert_result(CGDisplay::active_displays())
            .chain_err(|| "Unable to list active displays")?;
        for (i, &display_id) in displays.iter().enumerate() {
            let current_display_mode =
                ScreenResolution::get_current_mode_for_display(i as DisplayIndex, display_id)?;
            ScreenResolution::all_display_modes(display_id)?
                .into_iter()
                .for_each(|cgmode| {
                    let io_flags = cgmode.io_flags();
                    if (io_flags & (kDisplayModeValidFlag | kDisplayModeSafeFlag)) != 0 {
                        let mut mode = Mode::from(i as DisplayIndex, cgmode);
                        mode.current = mode == current_display_mode;
                        modes.push(mode);
                    }
                });
        }
        modes.sort_unstable_by(|a, b| {
            a.display
                .cmp(&(b.display))
                .then(a.width.cmp(&(b.width)).reverse())
                .then(a.height.cmp(&(b.height)).reverse())
        });

        Ok(ScreenResolution { displays, modes })
    }

    fn get_current_mode_for_display(
        display_index: DisplayIndex,
        display_id: CGDirectDisplayID,
    ) -> Result<Mode> {
        let display = CGDisplay::new(display_id);
        display
            .display_mode()
            .map(|cgmode| Mode::from(display_index, cgmode))
            .map_or_else(
                || Err(format!("No current mode for display: {}", display_index).into()),
                |mode| Ok(mode),
            )
    }

    pub fn print_current_mode(&self, long: bool, output: &mut io::Write) -> Result<()> {
        let current_modes: Vec<&Mode> = self.modes.iter().filter(|&mode| mode.current).collect();
        for mode in current_modes {
            mode.print_mode(long, output)?;
            writeln!(output, "");
        }
        Ok(())
    }

    pub fn parse_wanted_mode(mode: &str, display: DisplayIndex) -> Result<Mode> {
        // Parse: in the style of 1920x1200x32@0
        let re = Regex::new(r"(\d+)x(\d+)x(\d+)@(\d+)").chain_err(|| "Could not compile regex")?;
        let captures = re.captures(mode);
        captures.map_or_else(
            || Err(format!("Not a valid mode: {}", mode).into()),
            |caps| {
                Ok(Mode {
                    display: display,
                    cgmode: None,
                    width: caps.get(1).unwrap().as_str().parse().unwrap(),
                    height: caps.get(2).unwrap().as_str().parse().unwrap(),
                    pixel_width: 0,
                    pixel_height: 0,
                    refresh_rate: caps.get(4).unwrap().as_str().parse().unwrap(),
                    io_flags: 0,
                    bit_depth: caps.get(3).unwrap().as_str().parse().unwrap(),
                    current: false,
                })
            },
        )
    }

    fn configure_display(cgmode: &CGDisplayMode, display_id: CGDirectDisplayID) -> Result<()> {
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
    fn verify_current(
        &self,
        mode: &Mode,
        display_index: DisplayIndex,
        _display_id: CGDirectDisplayID,
    ) -> bool {
        !self
            .modes
            .iter()
            .filter(|&m| m.current && m.display == display_index && mode == m)
            .next()
            .is_some()
    }

    pub fn set_current_mode(&self, mode: &str, display_index: DisplayIndex) -> Result<()> {
        println!("Setting mode: {}, display: {}", mode, display_index);
        let wanted_mode = ScreenResolution::parse_wanted_mode(mode, display_index)
            .chain_err(|| "Could not parse wanted mode")?;
        let display_id = self.displays.get(display_index as usize);
        if let Some(&display_id) = display_id {
            if self.verify_current(&wanted_mode, display_index, display_id) {
                let possible_index = self
                    .modes
                    .iter()
                    .enumerate()
                    .filter(|(_, ref mode)| **mode == wanted_mode)
                    .map(|(i, _)| i)
                    .next();

                if let Some(index) = possible_index {
                    let cgmode = &self.modes.get(index).unwrap().cgmode.as_ref();
                    ScreenResolution::configure_display(cgmode.unwrap(), display_id)
                        .chain_err(|| "Could not actually configure display")?;
                }
                Ok(())
            } else {
                Err("Wanted Mode is already current".into())
            }
        } else {
            Err(format!("Unable to set mode for display: {}", display_index).into())
        }
    }

    fn all_display_modes(display_id: CGDirectDisplayID) -> Result<Vec<CGDisplayMode>> {
        let value = CFNumber::from(1);
        let key =
            unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef()).map_or_else(
            || Err("No display modes for display".into()),
            |cgmodes| Ok(cgmodes),
        )
    }

    pub fn list_modes(&self, long: bool, output: &mut io::Write) -> Result<()> {
        for mode in self.modes.iter() {
            mode.print_mode(long, output)
                .chain_err(|| "Could not list modes")?;
            writeln!(output, "")?;
        }
        Ok(())
    }

    pub fn set_from_list_modes(&self, long: bool, display_index: DisplayIndex) -> Result<()> {
        let mut selections = Vec::<String>::new();
        let mut set_strings = Vec::<String>::new();
        for mode in self.modes.iter() {
            let mut output = Vec::<u8>::new();
            mode.print_mode(long, &mut output)
                .chain_err(|| "Could not list modes")?;
            let selection = String::from_utf8(output).unwrap();
            selections.push(selection);
            set_strings.push(mode.for_select());
        }
        let selections_as_str: Vec<&str> = selections.iter().map(AsRef::as_ref).collect();
        //let selections_as_str = selections.into_iter().map(|sel| -> sel.as_str()).collect();

        let selection = Select::new()
            //.item(">>>>   Cancel")
            .items(&selections_as_str.as_slice())
            .default(0)
            .interact_opt()
            .unwrap();
        match selection {
            Some(selection) => {
                println!("Setting mode {}", set_strings[selection]);
                self.set_current_mode(set_strings[selection].as_str(), display_index)?;
            }
            _ => {
                println!("You cancelled");
            }
        }
        Ok(())
    }
}

fn run() -> Result<()> {
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
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
                    Arg::with_name("long")
                        .long("long")
                        .short("l")
                        .help("Shows more details on the displayed resolutions")
                        .required(false)
                        .takes_value(false),
                ),
        ).subcommand(
            SubCommand::with_name("get")
                .about("Get current active resution for current display")
                .arg(
                    Arg::with_name("long")
                        .long("long")
                        .short("l")
                        .help("Shows more details on the current resolution")
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
                    Arg::with_name("text-resolution")
                        .value_name("RESOLUTION")
                        .help("Resolution string in the form of WxHxP@R (e.g.: 1920x1200x32@0)")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("interactive-resolution")
                        .long("interactive")
                        .short("i")
                        .help("Will allow to choose resolution interactively")
                        .required(false),
                ).group(
                    ArgGroup::with_name("resolution")
                        .args(&["text-resolution", "interactive-resolution"])
                        .required(true),
                ),
        ).get_matches();

    let screen_resolution = ScreenResolution::new()?;
    match matches.subcommand() {
        ("list", Some(sub_m)) => {
            let long = sub_m.is_present("long");
            screen_resolution.list_modes(long, &mut output)
        }
        ("get", Some(sub_m)) => {
            let long = sub_m.is_present("long");
            screen_resolution.print_current_mode(long, &mut output)
        }
        ("set", Some(sub_m)) => {
            let display = sub_m
                .value_of("display")
                .unwrap_or("0")
                .parse::<DisplayIndex>()
                .unwrap_or(0);
            if sub_m.value_of("text-resolution").is_some() {
                screen_resolution.set_current_mode(sub_m.value_of("resolution").unwrap(), display)
            } else if sub_m.is_present("interactive-resolution") {
                screen_resolution.set_from_list_modes(false, display)
            } else {
                Err("Not a valid option".into())
            }
        }
        _ => Ok(()),
    }
}

quick_main!(run);
