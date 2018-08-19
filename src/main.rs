extern crate clap;
extern crate core_foundation;
extern crate core_graphics;
extern crate libc;

#[macro_use]
extern crate foreign_types;
use foreign_types::ForeignType;

use std::ops::Deref;
use std::ptr::null;

use core_foundation::array::CFArray;
use core_foundation::array::CFArrayRef;
use core_foundation::base::{CFRetain, TCFType};
use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::display::{CGDirectDisplayID, CGDisplay};

use clap::{App, Arg, SubCommand};

mod cg {
    pub enum CGDisplayMode {}
    pub type CGDisplayModeRef = *mut CGDisplayMode;
}

foreign_type! {
    #[doc(hidden)]
    type CType = cg::CGDisplayMode;
    fn drop = CGDisplayModeRelease;
    fn clone = |p| CFRetain(p as *const _) as *mut _;
    pub struct CGDisplayMode;
    pub struct CGDisplayModeRef;
}

fn list_modes() {
    println!(
        "Active display count: {}",
        CGDisplay::active_display_count().unwrap()
    );
    let vec = CGDisplay::active_displays().unwrap();
    for display_id in vec.clone() {
        println!("active display id: {}", display_id);
    }

    let main_display = CGDisplay::main();
    let main_display_mode = main_display.display_mode().unwrap();
    println!("main_display_mode width {}", main_display_mode.width());

    let array_opt: Option<CFArray> = unsafe {
        let array_ref = CGDisplayCopyAllDisplayModes(vec[0], null());
        if array_ref != null() {
            Some(CFArray::wrap_under_create_rule(array_ref))
        } else {
            None
        }
    };
    let modes = array_opt.unwrap();

    modes.into_iter().for_each(|value0| {
        let x = *value0.deref() as *mut cg::CGDisplayMode;
        let cgmode = unsafe { CGDisplayMode::from_ptr(x) };
        println!(
            "Display {}x{}, pixel {}x{}, refresh rate: {}, flags: {}, pixel encoding: {}",
            cgmode.width(),
            cgmode.height(),
            cgmode.pixel_width(),
            cgmode.pixel_height(),
            cgmode.refresh_rate(),
            cgmode.io_flags(),
            cgmode.pixel_encoding()
        );
    });

    // println!("Number of modes {}", modes.len());
    // // let values = modes.into_untyped();
    // let value_opt_0 = modes.get(0);
    // let value0 = value_opt_0.unwrap();
    // let x = *value0.deref() as *mut cg::CGDisplayMode;
    // let cgmode = unsafe { CGDisplayMode::from_ptr(x) };
    // println!("{}", cgmode.width());
    //println!("{:?}", modes.get(0).unwrap().deref())
    // for mode in modes.iter() {
    //     println!("mode {}", mode);
    // }
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
            SubCommand::with_name("get")
                .about("Get current active resution for current display (TODO)"),
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
        ("get", Some(_sub_m)) => println!("Get"),
        _ => {}
    }
}

impl CGDisplayMode {
    #[inline]
    pub fn height(&self) -> u64 {
        unsafe { CGDisplayModeGetHeight(self.as_ptr()) as u64 }
    }

    #[inline]
    pub fn width(&self) -> u64 {
        unsafe { CGDisplayModeGetWidth(self.as_ptr()) as u64 }
    }

    #[inline]
    pub fn pixel_height(&self) -> u64 {
        unsafe { CGDisplayModeGetPixelHeight(self.as_ptr()) as u64 }
    }

    #[inline]
    pub fn pixel_width(&self) -> u64 {
        unsafe { CGDisplayModeGetPixelWidth(self.as_ptr()) as u64 }
    }

    #[inline]
    pub fn refresh_rate(&self) -> f64 {
        unsafe { CGDisplayModeGetRefreshRate(self.as_ptr()) }
    }

    #[inline]
    pub fn io_flags(&self) -> u32 {
        unsafe { CGDisplayModeGetIOFlags(self.as_ptr()) as u32 }
    }

    #[inline]
    pub fn pixel_encoding(&self) -> CFString {
        unsafe { CFString::wrap_under_create_rule(CGDisplayModeCopyPixelEncoding(self.as_ptr())) }
    }

    // pub fn bitDepth(mode: CGDisplayModeRef)->_u32 {
    //     let  depth = 0;
    //     CFStringRef pixelEncoding = CGDisplayModeCopyPixelEncoding(mode);
    //     // my numerical representation for kIO16BitFloatPixels and kIO32bitFloatPixels
    //     // are made up and possibly non-sensical
    //     if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(kIO32BitFloatPixels), kCFCompareCaseInsensitive)) {
    //         depth = 96;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(kIO64BitDirectPixels), kCFCompareCaseInsensitive)) {
    //         depth = 64;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(kIO16BitFloatPixels), kCFCompareCaseInsensitive)) {
    //         depth = 48;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(IO32BitDirectPixels), kCFCompareCaseInsensitive)) {
    //         depth = 32;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(kIO30BitDirectPixels), kCFCompareCaseInsensitive)) {
    //         depth = 30;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(IO16BitDirectPixels), kCFCompareCaseInsensitive)) {
    //         depth = 16;
    //     } else if (kCFCompareEqualTo == CFStringCompare(pixelEncoding, CFSTR(IO8BitIndexedPixels), kCFCompareCaseInsensitive)) {
    //         depth = 8;
    //     }
    //     CFRelease(pixelEncoding);
    //     return depth;
    // }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGDisplayModeRelease(mode: cg::CGDisplayModeRef);

    pub fn CGDisplayCopyDisplayMode(display: CGDirectDisplayID) -> cg::CGDisplayModeRef;
    pub fn CGDisplayModeGetHeight(mode: cg::CGDisplayModeRef) -> libc::size_t;
    pub fn CGDisplayModeGetWidth(mode: cg::CGDisplayModeRef) -> libc::size_t;
    pub fn CGDisplayModeGetPixelHeight(mode: cg::CGDisplayModeRef) -> libc::size_t;
    pub fn CGDisplayModeGetPixelWidth(mode: cg::CGDisplayModeRef) -> libc::size_t;
    pub fn CGDisplayModeGetRefreshRate(mode: cg::CGDisplayModeRef) -> libc::c_double;
    pub fn CGDisplayModeGetIOFlags(mode: cg::CGDisplayModeRef) -> libc::uint32_t;
    pub fn CGDisplayModeCopyPixelEncoding(mode: cg::CGDisplayModeRef) -> CFStringRef;

    pub fn CGDisplayCopyAllDisplayModes(
        display: CGDirectDisplayID,
        options: CFDictionaryRef,
    ) -> CFArrayRef;
}
