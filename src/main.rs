extern crate clap;
extern crate core_foundation;
extern crate core_graphics;
extern crate libc;

use core_foundation::base::TCFType;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_graphics::display::{kCGDisplayShowDuplicateLowResolutionModes, CGDisplay, CGDisplayMode};

use clap::{App, SubCommand};

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

fn list_modes() {
    /*
        int value = 1;
    CFNumberRef number = CFNumberCreate( kCFAllocatorDefault, kCFNumberIntType, &value );
    CFStringRef key = kCGDisplayShowDuplicateLowResolutionModes;
    CFDictionaryRef options = CFDictionaryCreate( kCFAllocatorDefault, (const void **)&key, (const void **)&number, 1, NULL, NULL );

    */
    let value = CFNumber::from(1);
    let key = unsafe { CFString::wrap_under_get_rule(kCGDisplayShowDuplicateLowResolutionModes) };
    let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

    let displays = CGDisplay::active_displays().unwrap();

    displays.into_iter().enumerate().for_each(|(i, display)| {
        let array_opt: Option<Vec<CGDisplayMode>> =
            CGDisplayMode::all_display_modes(display, options.as_concrete_TypeRef());
        let modes = array_opt.unwrap();

        modes.into_iter().for_each(|cgmode| {
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
    })
}

fn main() {
    let matches = App::new("MacOS Screen Resolution Tool")
        .version("0.1.0")
        .author("Bernard Niset")
        .about("Allows to list, get and set screen resolutions.")
        .subcommand(
            SubCommand::with_name("list").about("List available resolutions for current display"),
        )
        .subcommand(
            SubCommand::with_name("get").about("Get current active resution for current display"),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set current active resution for current display (TODO)"),
        )
        .get_matches();
    match matches.subcommand() {
        ("list", Some(_sub_m)) => {
            list_modes();
        }
        ("get", Some(_sub_m)) => get_current_mode(),
        _ => {}
    }
}
