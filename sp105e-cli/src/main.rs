use anyhow::{anyhow, Result};

use args::{CliCommand, FixedColor};
use clap::Parser;
use sp105e::{client::LEDClient, commands::Command};

mod args;

#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = args::Cli::parse();

    let client = LEDClient::new(cli.adapter, cli.target).await?;
    let command = match cli.verb {
        CliCommand::Power => Command::Power,
        CliCommand::SetPixel { pixel } => Command::SetPixelType(pixel),
        CliCommand::SetOrder { order } => Command::SetColorOrder(order),
        CliCommand::SetColor { r, g, b } => Command::Color([r, g, b]),
        CliCommand::SetFixedColor { color } => match color {
            FixedColor::Red => Command::FixedRed,
            FixedColor::Green => Command::FixedGreen,
            FixedColor::Blue => Command::FixedBlue,
            FixedColor::White => Command::FixedWhite1,
            FixedColor::AltWhite => Command::FixedWhite2,
        },
        CliCommand::SetAnimation { id } => Command::Animation(id),
        CliCommand::Speed { up } => {
            if up > 0 {
                Command::SpeedUp
            } else {
                Command::SpeedDown
            }
        }
        CliCommand::Brightness { up } => {
            if up > 0 {
                Command::BrightnessUp
            } else {
                Command::BrightnessDown
            }
        }
    };

    client.send_cmd(&command).await?;
    Ok(())
}
