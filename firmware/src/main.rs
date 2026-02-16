#![feature(mpmc_channel)]

use std::sync::Arc;

use esp_idf_hal::{
    delay::FreeRtos,
    i2s::{
        config::{
            Config as ChannelConfig, DataBitWidth, SlotMode, StdClkConfig, StdConfig,
            StdGpioConfig, StdSlotConfig,
        },
        I2sDriver,
    },
    peripheral::Peripheral,
    prelude::*,
    spi::{config::DriverConfig, SpiDriver},
};
use global_channel::global_channel;

use crate::{bluetooth::Bluetooth, codec::Codec};

mod bluetooth;
mod codec;

const I2S_FREQUENCY: u32 = 44100;

global_channel!(bluetooth_audio, Some(32), Vec<u8>);

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let spi_driver = Arc::new(SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio32,
        peripherals.pins.gpio22,
        Option::<esp_idf_hal::gpio::AnyIOPin>::None,
        &DriverConfig::new(),
    )?);

    let i2s_config = StdConfig::new(
        ChannelConfig::default(),
        StdClkConfig::from_sample_rate_hz(I2S_FREQUENCY),
        StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Stereo),
        StdGpioConfig::default(),
    );

    let ext_int_i2s = I2sDriver::new_std_bidir(
        peripherals.i2s0,
        &i2s_config,
        peripherals.pins.gpio16,
        peripherals.pins.gpio15,
        peripherals.pins.gpio17,
        Some(peripherals.pins.gpio0),
        peripherals.pins.gpio5,
    )?;

    let ext_int_codec = Codec::new(
        spi_driver.clone(),
        peripherals.pins.gpio4,
        ext_int_i2s,
        bluetooth_audio_rx(),
    )
    .unwrap();

    let bluetooth = Bluetooth::new(peripherals.modem, bluetooth_audio_tx())?;

    std::thread::spawn(|| ext_int_codec.entrypoint());
    std::thread::spawn(|| bluetooth.entrypoint());

    loop {
        FreeRtos::delay_ms(20);
    }
}
