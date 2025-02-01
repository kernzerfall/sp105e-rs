use clap::*;
use sp105e::commands::{ColorOrder, PixelType};

#[derive(Clone, Debug, clap::Subcommand)]
pub(super) enum CliCommand {
    /// Toggle power state
    Power,
    /// Set the pixel type
    SetPixel { pixel: PixelType },
    /// Set the number of pixels. Range = [1, 2048].
    SetNumber { num: u16 },
    /// Set the color order.
    SetOrder { order: ColorOrder },
    /// Set a custom color (rgb).
    SetColor { r: u8, g: u8, b: u8 },
    /// Set one of the fixed colors.
    SetFixedColor { color: FixedColor },
    /// Start an animation (0 = auto).
    SetAnimation { id: u8 },
    /// Set the speed. Range = [0, 6].
    Speed { up: u8 },
    /// Set the brightness. Range = [0, 6].
    Brightness { up: u8 },
    /// Get status information from the controller.
    Status,
}

#[derive(Clone, Debug, ValueEnum)]
pub(super) enum FixedColor {
    Red,
    Green,
    Blue,
    White,
    AltWhite,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub verb: CliCommand,

    /// MAC of the Bluetooth adapter to use on the Host.
    #[arg(short, long, value_name = "01:23:45:67:89:ab")]
    pub adapter: Option<String>,

    /// MAC of the target SP105E device.
    #[arg(short, long, value_name = "01:23:45:67:89:ab")]
    pub target: String,
}
