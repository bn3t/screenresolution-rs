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
    CGConfigureOption, CGDisplay, CGDisplayMode,
};

use clap::{App, AppSettings, Arg, ArgGroup, SubCommand};

use dialoguer::Select;

mod errors;
mod mode;

type DisplayId = u32;

use errors::*;
use mode::*;

fn get_current_mode_for_display(
    display_index: DisplayIndex,
    display_id: DisplayId,
) -> Result<Mode> {
    let display = CGDisplay::new(display_id);
    display
        .display_mode()
        .map(|cgmode| Mode::from(display_index, &cgmode))
        .map_or_else(
            || Err(format!("No current mode for display: {}", display_index).into()),
            |mode| Ok(mode),
        )
}

fn print_current_mode(short: bool, output: &mut io::Write) -> Result<()> {
    println!(
        "Active display count: {}",
        convert_result(CGDisplay::active_display_count())
            .chain_err(|| "Could not get active display count")?
    );
    let displays = convert_result(CGDisplay::active_displays())
        .chain_err(|| "Could not list active displays")?;
    let displays_enumerated = displays.into_iter().enumerate();
    for (i, display_id) in displays_enumerated {
        let mode = get_current_mode_for_display(i as DisplayIndex, display_id)?;
        mode.print_mode(short, output)?;
    }
    Ok(())
}

fn parse_wanted_mode(mode: &str, display: DisplayIndex) -> Result<Mode> {
    // Parse: in the style of 1920x1200x32@0
    let re = Regex::new(r"(\d+)x(\d+)x(\d+)@(\d+)").chain_err(|| "Could not compile regex")?;
    let captures = re.captures(mode);
    captures.map_or_else(
        || Err(format!("Not a valid mode: {}", mode).into()),
        |caps| {
            Ok(Mode {
                display: display,
                width: caps.get(1).unwrap().as_str().parse().unwrap(),
                height: caps.get(2).unwrap().as_str().parse().unwrap(),
                pixel_width: 0,
                pixel_height: 0,
                refresh_rate: caps.get(4).unwrap().as_str().parse().unwrap(),
                io_flags: 0,
                bit_depth: caps.get(3).unwrap().as_str().parse().unwrap(),
            })
        },
    )
}

fn configure_display(cgmode: &CGDisplayMode, display_id: DisplayId) -> Result<()> {
    println!("configure_display");
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

fn get_display_id_from_display_index(display_index: DisplayIndex) -> Result<DisplayId> {
    let display_ids = CGDisplay::active_displays().unwrap();
    display_ids
        .into_iter()
        .enumerate()
        .filter(|(i, _)| *i as DisplayIndex == display_index)
        .map(|(_, display_id)| display_id)
        .next()
        .map_or_else(
            || Err(format!("Not a valid display: {}", display_index).into()),
            |display_id| Ok(display_id),
        )
}

fn set_current_mode(mode: &str, display_index: DisplayIndex) -> Result<()> {
    {
        println!("optain all modes");
        let all_modes = obtain_all_modes_for_all_displays();
        for mode in all_modes {
            println!("mode: {:?}", mode);
        }
    }
    println!("Setting mode: {}, display: {}", mode, display_index);
    let wanted_mode =
        parse_wanted_mode(mode, display_index).chain_err(|| "Could not parse wanted mode")?;
    let display_id = get_display_id_from_display_index(display_index)?;
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
}

fn all_display_modes(display_id: DisplayId) -> Result<Vec<CGDisplayMode>> {
    let value = CFNumber::from(1);
    let key = unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    CGDisplayMode::all_display_modes(display_id, options.as_concrete_TypeRef()).map_or_else(
        || Err("No display modes for display".into()),
        |cgmodes| Ok(cgmodes),
    )
}

fn obtain_all_modes_for_all_displays() -> Result<Vec<Mode>> {
    let mut result: Vec<Mode> = Vec::with_capacity(50);

    let display_ids = convert_result(CGDisplay::active_displays())
        .chain_err(|| "Unable to list active displays")?;

    let display_ids_enumerate = display_ids.into_iter().enumerate();
    for (i, display_id) in display_ids_enumerate {
        let modes: Vec<CGDisplayMode> = all_display_modes(display_id)?;

        modes.into_iter().for_each(|cgmode| {
            let io_flags = cgmode.io_flags();
            if (io_flags & (kDisplayModeValidFlag | kDisplayModeSafeFlag)) != 0 {
                result.push(Mode::from(i as DisplayIndex, &cgmode));
            }
        });
    }
    result.sort_unstable_by(|a, b| {
        a.display
            .cmp(&(b.display))
            .then(a.width.cmp(&(b.width)).reverse())
            .then(a.height.cmp(&(b.height)).reverse())
    });
    Ok(result)
}

fn list_modes(short: bool, output: &mut io::Write) -> Result<()> {
    let all_modes = obtain_all_modes_for_all_displays()?;
    for mode in all_modes {
        mode.print_mode(short, output)
            .chain_err(|| "Could not list modes")?;
        writeln!(output, "")?;
    }
    Ok(())
}

fn set_from_list_modes(short: bool, display_index: DisplayIndex) -> Result<()> {
    let all_modes = obtain_all_modes_for_all_displays()?;

    let mut selections = Vec::<String>::new();
    let mut set_strings = Vec::<String>::new();
    for mode in all_modes.iter() {
        let mut output = Vec::<u8>::new();
        mode.print_mode(short, &mut output)
            .chain_err(|| "Could not list modes")?;
        let selection = String::from_utf8(output).unwrap();
        selections.push(selection);
        set_strings.push(mode.for_select());
    }
    let selections_as_str: Vec<&str> = selections.iter().map(AsRef::as_ref).collect();
    //let selections_as_str = selections.into_iter().map(|sel| -> sel.as_str()).collect();

    let selection = Select::new()
        .item(">>>>   Cancel")
        .items(&selections_as_str.as_slice())
        .interact()
        .unwrap();
    if selection > 0 {
        println!("Setting mode {}", set_strings[selection - 1]);
        set_current_mode(set_strings[selection - 1].as_str(), display_index)?;
    } else {
        println!("You cancelled");
    }
    // if let Result(selection) = selection {
    // }
    Ok(())
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
                    Arg::with_name("text-resolution")
                        .value_name("RESOLUTION")
                        .help("Resolution string in the form of WxHxP@R (e.g.: 1920x1200x32@0)")
                        .required(false)
                        .takes_value(true),
                ).arg(
                    Arg::with_name("interactive-resolution")
                        .short("i")
                        .help("Will allow to choose resolution interactively")
                        .required(false),
                ).group(
                    ArgGroup::with_name("resolution")
                        .args(&["text-resolution", "interactive-resolution"])
                        .required(true),
                ),
        ).get_matches();
    match matches.subcommand() {
        ("list", Some(sub_m)) => {
            let short = sub_m.is_present("short");
            list_modes(short, &mut output)
        }
        ("get", Some(sub_m)) => {
            let short = sub_m.is_present("short");
            print_current_mode(short, &mut output)
        }
        ("set", Some(sub_m)) => {
            let display = sub_m
                .value_of("display")
                .unwrap_or("0")
                .parse::<DisplayIndex>()
                .unwrap_or(0);
            if sub_m.value_of("text-resolution").is_some() {
                set_current_mode(sub_m.value_of("resolution").unwrap(), display)
            } else if sub_m.is_present("interactive-resolution") {
                set_from_list_modes(true, display)
            } else {
                Err("Not a valid option".into())
            }
        }
        _ => Ok(()),
    }
}

quick_main!(run);
