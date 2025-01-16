use std::str::FromStr;

use anyhow::{anyhow, Result};
use bluer::{gatt::remote::Characteristic, Address, Device, Session, Uuid};
use futures_util::{pin_mut, StreamExt};

use crate::commands::{
    Command, StatusResp, GATT_CHARACTERISTIC_UUID, GATT_SERVICE_UUID, STATUS_RETURN_LENGTH,
};

pub struct LEDClient {
    device: Device,
    characteristic: Characteristic,
}

impl LEDClient {
    async fn find_led_characteristic(device: &Device) -> Result<Characteristic> {
        let w_ser_uuid = Uuid::from_str(GATT_SERVICE_UUID)?;
        let w_chr_uuid = Uuid::from_str(GATT_CHARACTERISTIC_UUID)?;

        for service in device.services().await? {
            let uuid = service.uuid().await?;
            if uuid != w_ser_uuid {
                continue;
            }

            println!("Found wanted service UUID");

            for chr in service.characteristics().await? {
                let uuid = chr.uuid().await?;
                if uuid != w_chr_uuid {
                    continue;
                }

                println!("Found wanted characteristic UUID");

                println!("{:#?}", chr.all_properties().await?);

                return Ok(chr);
            }
        }

        return Err(anyhow! {"device does not have the required gatt service/characteristic"});
    }

    async fn ensure_device_connected(device: &Device) -> Result<()> {
        if !device.is_connected().await? {
            let mut retry = 3;
            loop {
                println!("Trying to connect\n");
                if device.connect().await.is_ok() {
                    break;
                }

                retry -= 1;

                if retry == 0 {
                    return Err(anyhow! {"failed to connect to device"});
                }

                std::thread::sleep(std::time::Duration::new(3, 0));
            }
        }

        Ok(())
    }

    async fn ensure_connected(&self) -> Result<()> {
        Self::ensure_device_connected(&self.device).await
    }

    pub async fn new(adapter_name: Option<String>, target_mac: String) -> Result<Self> {
        let session = Session::new().await?;
        let adapter = match adapter_name {
            Some(name) => session.adapter(name.as_str())?,
            None => session.default_adapter().await?,
        };

        let device = adapter.device(Address::from_str(&target_mac)?)?;
        if !device.is_connected().await? {
            Self::ensure_device_connected(&device).await?;
        } else {
            println!("Already connected!");
        }

        let characteristic = Self::find_led_characteristic(&device).await?;

        Ok(Self {
            device,
            characteristic,
        })
    }

    pub async fn send_cmd(&self, command: &Command) -> Result<()> {
        self.ensure_connected().await?;
        self.characteristic.write(&*command.buf()).await?;

        Ok(())
    }

    pub async fn send_cmd_with_callback(
        &self,
        command: &Command,
    ) -> Result<impl futures_core::stream::Stream<Item = Vec<u8>>> {
        self.ensure_connected().await?;

        let ind = self.characteristic.notify().await?;
        self.characteristic.write(&*command.buf()).await?;

        Ok(ind)
    }

    pub async fn get_status(&self) -> Result<StatusResp> {
        let ret = self.send_cmd_with_callback(&Command::Status).await?;
        let mut res: Vec<u8> = Vec::new();
        pin_mut!(ret);
        while res.len() < STATUS_RETURN_LENGTH as usize {
            match ret.next().await {
                Some(value) => res.extend(value),
                None => {
                    println!("notification session terminated prematurely");
                    break;
                }
            }
        }

        res.try_into()
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::min, time::Duration};

    use super::*;

    // Change this mac to test with your own device
    static TEST_MAC: &str = "69:96:06:04:0C:B1";
    static DUR: Duration = Duration::new(1, 0);

    #[tokio::test]
    async fn connection() {
        let _ = LEDClient::new(None, TEST_MAC.to_string()).await.unwrap();
    }

    #[tokio::test]
    async fn fixed_rgb() {
        let c = LEDClient::new(None, TEST_MAC.to_string()).await.unwrap();

        for command in [
            Command::FixedRed,
            Command::FixedGreen,
            Command::FixedBlue,
            Command::FixedWhite1,
            Command::FixedWhite2,
        ] {
            c.send_cmd(&command).await.unwrap();
            std::thread::sleep(DUR);
        }
    }

    #[tokio::test]
    async fn custom_rgb() {
        let c = LEDClient::new(None, TEST_MAC.to_string()).await.unwrap();

        for i in 0..10 {
            let command = Command::Color([
                255 - 25 * i,
                2 * i * i,
                min((1f64 / ((i * i + 1) as f64)) as u8, 255),
            ]);
            c.send_cmd(&command).await.unwrap();
            std::thread::sleep(DUR);
        }
    }

    #[tokio::test]
    async fn animation() {
        let c = LEDClient::new(None, TEST_MAC.to_string()).await.unwrap();

        for anim in [0, 32, 54, 96] {
            let command = Command::Animation(anim);
            c.send_cmd(&command).await.unwrap();
            std::thread::sleep(10 * DUR);
        }
    }
}
