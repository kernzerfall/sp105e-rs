use clap::*;
use sp105e::commands::{ColorOrder, PixelType};

#[derive(Clone, Debug, clap::Subcommand)]
pub(super) enum CliCommand {
    Power,
    SetPixel { pixel: PixelType },
    SetOrder { order: ColorOrder },
    SetColor { r: u8, g: u8, b: u8 },
    SetFixedColor { color: FixedColor },
    SetAnimation { id: u8 },
    Speed { up: u8 },
    Brightness { up: u8 },
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
    #[arg(short, long)]
    pub adapter: Option<String>,
    #[arg(short, long)]
    pub target: String,
}
