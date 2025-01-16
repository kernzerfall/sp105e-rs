use std::mem::transmute;

use anyhow::{anyhow, Result};

pub const COMMAND_BUF_LENGTH: usize = 5;
pub const COMMAND_PREFIX: u8 = 0x38;

// The status command (0x10) returns 8 bytes via a notification
pub const STATUS_RETURN_LENGTH: u8 = 8;

pub const GATT_SERVICE_UUID: &str = "0000ffe0-0000-1000-8000-00805f9b34fb";
pub const GATT_CHARACTERISTIC_UUID: &str = "0000ffe1-0000-1000-8000-00805f9b34fb";

/// Commands which appear as suffixes in the buffer (<SUF>)
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum Command {
    /// Hello
    /// Structure: <PRE> 0 0 0 <SUF>
    /// Controller notifies "00 01 02 03 04 05 06 bf" back if OK
    Hello = 0xD5,

    /// Config
    /// Structure <PRE> 0 0 0 <SUF>
    /// Controller notifies back its status (can't decode it fully yet!)
    Status = 0x10,

    /// Toggles power
    /// Structure: <PRE> 0 0 0 <SUF>
    Power = 0xAA,

    /// Sets the number of pixels
    /// Structure: <PRE> [NUM >> 8] [NUM & 0xf] 0 <SUF>
    SetPixels(u16) = 0x2D,

    /// Sets the order of the colors
    /// Structure: <PRE> <CO> 0 0 <SUF>
    SetColorOrder(ColorOrder) = 0x3C,

    /// Sets the pixel type
    /// Structure: <PRE> <PT> 0 0 <SUF>
    SetPixelType(PixelType) = 0x1C,

    // This group sets a fixed color mode (for some reason this is distinct
    // from setting a fixed color in the custom RGB command).
    // Structure: <PRE> 0 0 0 <SUF>, the suffix is determined by the color.
    /// Fixed color mode red
    FixedRed = 0x12,
    /// Fixed color mode green
    FixedGreen = 0x18,
    /// Fixed color mode blue
    FixedBlue = 0x36,
    /// Fixed color mode white1
    FixedWhite1 = 0x3B,
    /// Fixed color mode white2
    FixedWhite2 = 0x56,

    /// Sets a preprogrammed animation mode (range 0x00-0xC8)
    /// Structure: <PRE> <MODE> 0 0 <SUF>
    Animation(u8) = 0x2C,

    /// Sets a custom RGB color
    /// Structure: <PRE> <C1> <C2> <C3> <SUF>
    /// The color order is determined by the mode (RGB, BGR, ...)
    Color([u8; 3]) = 0x1E,

    /// Adjusts the speed upwards by one step
    SpeedUp = 0x03,

    /// Adjusts the speed downwards by one step
    SpeedDown = 0x09,

    /// Adjusts the brightness upwards by one step
    BrightnessUp = 0x2A,

    /// Adjusts the brightness downwards by one step
    BrightnessDown = 0x28,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum ColorOrder {
    #[default]
    RGB,
    RBG,
    GRB,
    GBR,
    BRG,
    BGR,
}

#[derive(PartialEq, Eq, Debug, Clone, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[repr(u8)]
pub enum PixelType {
    SM16703,
    TM1804,
    USC1903,
    WS2811,
    WS2801,
    SK6812,
    SK6812RGBW,
    LPD6803,
    LPD8806,
    #[default]
    APA102,
    APA105,
    TM1814,
    TM1914,
    TM1913,
    P9813,
    INK1003,
    DMX512,
    P943S,
    P9411,
    P9412,
    P9413,
    P9414,
    TX1812,
    TX1813,
    GS8206,
    GS8208,
    SK9822,
}

/// Struct representation of the bytes returned by the status command
/// The order of the `u8`s in the struct corresponds directly to the
/// bytes.
#[derive(PartialEq, Eq, Debug, Clone)]
#[repr(C)]
pub struct StatusResp {
    /// Power state of the strip (0=off, 1=on)
    pub power: u8,

    /// Current mode of the controller
    /// - [00-c8]    => animation
    /// - c9         => custom color (no indication as to _what_ color!)
    /// - ca, cb, cc => red, green, blue (FixedColor)
    /// - cd, ce     => white1, white2
    pub mode: Command,

    /// Speed of the animation. Is also set on non-animation modes.
    /// Range: [0, 6]
    pub speed: u8,

    /// Brightness
    /// Range: [0, 6]
    pub brightness: u8,

    /// Pixel type
    /// Range: see `enum PixelType`
    pub pixel_type: PixelType,

    /// Color order
    /// Range: see `enum ColorOrder`
    pub color_order: ColorOrder,

    /// Rest bytes in status message (function unknown)
    /// Always seem to be 0x01 0xf4 (maybe some controller ID?)
    pub _unknown: [u8; 2],
}

impl TryFrom<Vec<u8>> for StatusResp {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 8 {
            return Err(anyhow!("status vector has wrong size"));
        }

        let [power, mode_v, speed, brightness, pixel_type_v, color_order_v, u1, u2]: [u8] =
            value[..]
        else {
            return Err(anyhow!("could not unpack status vector!"));
        };

        // interpret *_v into their enum variants
        let mode = if mode_v <= 0xc8 {
            Command::Animation(mode_v)
        } else {
            match mode_v {
                0xc9 => Command::Color([0, 0, 0]),
                0xca => Command::FixedRed,
                0xcb => Command::FixedGreen,
                0xcc => Command::FixedBlue,
                0xcd => Command::FixedWhite1,
                0xce => Command::FixedWhite2,
                _ => return Err(anyhow!("unknown mode {mode_v}")),
            }
        };

        if !(0..=PixelType::SK9822 as u8).contains(&pixel_type_v) {
            return Err(anyhow!("pixel type {pixel_type_v} is not known"));
        }

        // SAFETY: we have already checked that the enum has this value!
        let pixel_type: PixelType = unsafe { transmute(pixel_type_v) };

        if !(0..=ColorOrder::BGR as u8).contains(&color_order_v) {
            return Err(anyhow!("pixel type {color_order_v} is not known"));
        }

        // SAFETY: we have already checked that the enum has this value!
        let color_order: ColorOrder = unsafe { transmute(color_order_v) };

        let _unknown = [u1, u2];

        Ok(StatusResp {
            power,
            mode,
            speed,
            brightness,
            pixel_type,
            color_order,
            _unknown,
        })
    }
}

impl Command {
    pub fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    #[no_mangle]
    pub extern "C" fn buf(&self) -> Box<[u8; COMMAND_BUF_LENGTH]> {
        let inner_bytes = match self {
            Command::SetPixels(pixels) => {
                let hi = (pixels >> 8) as u8;
                let lo: u8 = (pixels & 0xff) as u8;
                // TODO: check if pixels is less than 2048 (controller limit)
                [hi, lo, 0]
            }

            Command::Color(colors) => *colors,

            Command::Animation(animation) => [*animation, 0, 0],

            // TODO: check value
            Command::SetColorOrder(co) => [co.clone() as u8, 0, 0],

            // TODO: check value
            Command::SetPixelType(pt) => [pt.clone() as u8, 0, 0],

            // For commands that don't need inner bytes
            _ => [0, 0, 0],
        };

        Box::new([
            COMMAND_PREFIX,
            inner_bytes[0],
            inner_bytes[1],
            inner_bytes[2],
            self.discriminant(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_cmd() {
        assert_eq!(*Command::Power.buf(), [COMMAND_PREFIX, 0, 0, 0, 0xAA]);
        assert_eq!(*Command::FixedRed.buf(), [COMMAND_PREFIX, 0, 0, 0, 0x36]);
        assert_eq!(*Command::SpeedUp.buf(), [COMMAND_PREFIX, 0, 0, 0, 0x03]);
    }

    #[test]
    fn dynamic_color() {
        let rgb = [0x12, 0x34, 0x56];
        let result = Command::Color(rgb.clone()).buf();

        assert_eq!(*result, [COMMAND_PREFIX, rgb[0], rgb[1], rgb[2], 0x1E]);
    }

    #[test]
    fn dynamic_led_num() {
        let leds: u16 = 0x1234;

        let leds_hi = (leds >> 8) as u8;
        let leds_lo = (leds & 0xff) as u8;

        let result = Command::SetPixels(leds).buf();

        assert_eq!(*result, [COMMAND_PREFIX, leds_hi, leds_lo, 0, 0x2D]);
    }

    #[test]
    fn dynamic_order() {
        let order = ColorOrder::GRB;
        let ordinal = 2;

        assert_eq!(order.clone() as u8, ordinal);

        let result = Command::SetColorOrder(order).buf();

        assert_eq!(*result, [COMMAND_PREFIX, ordinal, 0, 0, 0x3C]);
    }
}
