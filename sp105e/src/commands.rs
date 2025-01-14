const COMMAND_BUF_LENGTH: usize = 5;
const COMMAND_PREFIX: u8 = 0x3B;

/// Commands which appear as suffixes in the buffer (<SUF>)
#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Command {
    /// Toggles power
    /// Structure: <PRE> 0 0 0 <SUF>
    Power = 0xAA,

    /// Sets the number of pixels
    /// Structure: <PRE> [NUM >> 8] [NUM & 0xf] 0 <SUF>
    SetPixels(u16) = 0x2D,

    /// Sets the order of the colors
    SetColorOrder(ColorOrder) = 0x3C,

    // This group sets a fixed color mode (for some reason this is distinct
    // from setting a fixed color in the custom RGB command).
    // Structure: <PRE> 0 0 0 <SUF>, the suffix is determined by the color.
    /// Fixed color mode red
    FixedRed = 0x36,
    /// Fixed color mode green
    FixedGreen = 0x18,
    /// Fixed color mode blue
    FixedBlue = 0x12,
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

impl Command {
    fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }

    pub fn buf(&self) -> [u8; COMMAND_BUF_LENGTH] {
        let inner_bytes = match self {
            Command::SetPixels(pixels) => {
                let hi = (pixels >> 8) as u8;
                let lo: u8 = (pixels & 0xff) as u8;
                // TODO: check if pixels is less than 2048 (controller limit)
                [hi, lo, 0]
            }

            // TODO: take the controller's mode into consideration?
            Command::Color(colors) => *colors,

            Command::SetColorOrder(co) => [co.clone() as u8, 0, 0],

            // For commands that don't need inner bytes
            _ => [0, 0, 0],
        };

        [
            COMMAND_PREFIX,
            inner_bytes[0],
            inner_bytes[1],
            inner_bytes[2],
            self.discriminant(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_cmd_test() {
        assert_eq!(Command::Power.buf(), [COMMAND_PREFIX, 0, 0, 0, 0xAA]);
        assert_eq!(Command::FixedRed.buf(), [COMMAND_PREFIX, 0, 0, 0, 0x36]);
        assert_eq!(Command::SpeedUp.buf(), [COMMAND_PREFIX, 0, 0, 0, 0x03]);
    }

    #[test]
    fn dynamic_color_test() {
        let rgb = [0x12, 0x34, 0x56];
        let result = Command::Color(rgb.clone()).buf();

        assert_eq!(result, [COMMAND_PREFIX, rgb[0], rgb[1], rgb[2], 0x1E]);
    }

    #[test]
    fn dynamic_led_num_test() {
        let leds: u16 = 0x1234;

        let leds_hi = (leds >> 8) as u8;
        let leds_lo = (leds & 0xff) as u8;

        let result = Command::SetPixels(leds).buf();

        assert_eq!(result, [COMMAND_PREFIX, leds_hi, leds_lo, 0, 0x2D]);
    }

    #[test]
    fn dynamic_order_test() {
        let order = ColorOrder::GRB;
        let ordinal = 2;

        assert_eq!(order.clone() as u8, ordinal);

        let result = Command::SetColorOrder(order).buf();

        assert_eq!(result, [COMMAND_PREFIX, ordinal, 0, 0, 0x3C]);
    }
}
