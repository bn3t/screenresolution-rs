use core_graphics::display::CGDisplayMode;

pub type DisplayIndex = u8;

pub enum ScreenFormat {
    F16_9,
    F16_10,
    F4_3,
}
pub struct Mode {
    pub display: DisplayIndex,
    pub width: u64,
    pub height: u64,
    pub pixel_width: u64,
    pub pixel_height: u64,
    pub refresh_rate: f64,
    pub io_flags: u32,
    pub bit_depth: usize,
}

impl Mode {
    pub fn from(display: DisplayIndex, cgmode: &CGDisplayMode) -> Mode {
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

    pub fn is_hdpi(&self) -> bool {
        self.width != self.pixel_width || self.height != self.pixel_height
    }

    pub fn screen_format(&self) -> ScreenFormat {
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

    pub fn print_short(&self) {
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

    pub fn print_long(&self) {
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

    pub fn print_mode(&self, short: bool) {
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
