use core_graphics::display::CGDisplayMode;

use std::io;

use errors::*;

pub type DisplayIndex = u8;

pub enum ScreenFormat {
    F16_9,
    F16_10,
    F4_3,
}

pub struct Mode {
    pub display: DisplayIndex,
    pub cgmode: Option<CGDisplayMode>,
    pub width: u64,
    pub height: u64,
    pub pixel_width: u64,
    pub pixel_height: u64,
    pub refresh_rate: f64,
    pub io_flags: u32,
    pub bit_depth: usize,
}

impl Mode {
    pub fn from(display: DisplayIndex, cgmode: CGDisplayMode) -> Mode {
        Mode {
            display: display,
            width: cgmode.width(),
            height: cgmode.height(),
            pixel_width: cgmode.pixel_width(),
            pixel_height: cgmode.pixel_height(),
            refresh_rate: cgmode.refresh_rate(),
            io_flags: cgmode.io_flags(),
            bit_depth: cgmode.bit_depth(),
            cgmode: Some(cgmode),
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

    fn print_short(&self, output: &mut io::Write) -> Result<()> {
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
        write!(
            output,
            "Display {}: {:15} - pixel {:15} - {:6} - {:6}",
            self.display, mode_str, mode_pixel, hidpi, screen_format
        ).chain_err(|| "Could not print long")?;
        Ok(())
    }

    fn print_long(&self, output: &mut io::Write) -> Result<()> {
        let hidpi = if self.is_hdpi() { "HiDPI" } else { "" };
        let screen_format = match self.screen_format() {
            ScreenFormat::F16_9 => "16:9",
            ScreenFormat::F16_10 => "16:10",
            ScreenFormat::F4_3 => "4:3",
        };
        write!(
            output,
            "Display {}: {}x{}, refresh rate: {}, bitDepth: {}, flags: 0x{:07X}, {}, {}",
            self.display,
            self.width,
            self.height,
            self.refresh_rate,
            self.bit_depth,
            self.io_flags,
            hidpi,
            screen_format
        ).chain_err(|| "Could not print long")?;
        Ok(())
    }

    pub fn print_mode(&self, long: bool, output: &mut io::Write) -> Result<()> {
        match long {
            false => self.print_short(output)?,
            true => self.print_long(output)?,
        };
        Ok(())
    }

    pub fn for_select(&self) -> String {
        format!(
            "{}x{}x{}@{}",
            self.width, self.height, self.bit_depth, self.refresh_rate
        )
    }
}

impl PartialEq for Mode {
    fn eq(&self, other: &Mode) -> bool {
        self.display == other.display
            && self.width == other.width
            && self.height == other.height
            && self.bit_depth == other.bit_depth
            && self.refresh_rate == other.refresh_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_eq_equals() {
        let mode1 = Mode {
            display: 0,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 0,
            pixel_height: 0,
            refresh_rate: 75.0,
            io_flags: 0,
            bit_depth: 32,
        };
        let mode2 = Mode {
            display: 0,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 0,
            pixel_height: 0,
            refresh_rate: 75.0,
            io_flags: 0,
            bit_depth: 32,
        };
        assert_eq!(true, mode1 == mode2);
    }

    #[test]
    fn partial_eq_not_equals() {
        let mode1 = Mode {
            display: 0,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 0,
            pixel_height: 0,
            refresh_rate: 0.0,
            io_flags: 0,
            bit_depth: 0,
        };
        let mode2 = Mode {
            display: 0,
            cgmode: None,
            width: 800,
            height: 640,
            pixel_width: 0,
            pixel_height: 0,
            refresh_rate: 0.0,
            io_flags: 0,
            bit_depth: 0,
        };
        assert_eq!(false, mode1 == mode2);
    }

    #[test]
    fn print_mode_short() {
        let mode1 = Mode {
            display: 1,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 1024,
            pixel_height: 768,
            refresh_rate: 21.2,
            io_flags: 123,
            bit_depth: 32,
        };
        let mut vec = Vec::<u8>::new();

        mode1
            .print_mode(false, &mut vec)
            .expect("Error while testing print_short");

        assert_eq!(
            "Display 1: 800x600x32@21.2 - pixel 1024x768x32@21.2 - HiDPI  - 4:3   ",
            String::from_utf8(vec).unwrap().as_str()
        );
    }

    #[test]
    fn print_mode_long() {
        let mode1 = Mode {
            display: 1,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 1024,
            pixel_height: 768,
            refresh_rate: 21.2,
            io_flags: 123,
            bit_depth: 32,
        };
        let mut vec = Vec::<u8>::new();

        mode1
            .print_mode(true, &mut vec)
            .expect("Error while testing print_short");

        assert_eq!(
            "Display 1: 800x600, refresh rate: 21.2, bitDepth: 32, flags: 0x000007B, HiDPI, 4:3",
            String::from_utf8(vec).unwrap().as_str()
        );
    }

    #[test]
    fn mode_for_select() {
        let mode = Mode {
            display: 1,
            cgmode: None,
            width: 800,
            height: 600,
            pixel_width: 1024,
            pixel_height: 768,
            refresh_rate: 21.2,
            io_flags: 123,
            bit_depth: 32,
        };
        let actual = mode.for_select();
        assert_eq!("800x600x32@21.2", actual);
    }
}
