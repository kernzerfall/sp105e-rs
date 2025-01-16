use anyhow::Result;

use args::{CliCommand, FixedColor};
use clap::Parser;
use sp105e::{
    client::LEDClient,
    commands::{Command, StatusResp},
};

mod args;

async fn pretty_print_status(status: &StatusResp) -> Result<()> {
    println!("Power      : {:#04x}", status.power);
    println!(
        "Mode       : {:#04x} ({:?})",
        status.mode.discriminant(),
        status.mode
    );
    println!("Speed      : {:#04x}", status.speed);
    println!("Brightness : {:#04x}", status.brightness);
    println!(
        "PixelType  : {:#04x} ({:?})",
        status.pixel_type.clone() as u8,
        status.pixel_type
    );
    println!(
        "ColorOrder : {:#04x} ({:?})",
        status.color_order.clone() as u8,
        status.color_order
    );
    println!(
        "Unknown    : {:#04x} {:#04x}",
        status._unknown[0], status._unknown[1]
    );

    Ok(())
}

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
        CliCommand::GetState => Command::Status,
    };

    match command {
        Command::Status => {
            let status = client.get_status().await?;
            pretty_print_status(&status).await?;
        }
        c => client.send_cmd(&c).await?,
    }

    Ok(())
}
