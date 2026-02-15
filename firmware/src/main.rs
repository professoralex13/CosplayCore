use std::cell::RefCell;

use esp_idf_hal::{
    gpio::PinDriver,
    prelude::Peripherals,
    spi::{config::DriverConfig, SpiBusDriver, SpiConfig, SpiDriver},
};

use crate::codec::{
    spi::consts::{MicBoost, MAX_DAC_VOLUME, MAX_MIX_VOLUME},
    AudioChannel, Codec, PowerConfig,
};

mod codec;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let ext_int_csb = PinDriver::output(peripherals.pins.gpio4)?;

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio32,
        peripherals.pins.gpio22,
        Option::<esp_idf_hal::gpio::AnyIOPin>::None,
        &DriverConfig::new(),
    )?;

    let spi_bus = RefCell::new(SpiBusDriver::new(spi_driver, &SpiConfig::new())?);

    let mut codec1 = Codec::new(&spi_bus, ext_int_csb)?;

    codec1.set_power_management(PowerConfig {
        adc_left: false,
        adc_right: false,
        dac_left: true,
        dac_right: true,
        left_out_1: true,
        right_out_1: true,
        pga_left: true,
        pga_right: true,
    })?;

    codec1.set_input_volume(AudioChannel::Left, 50)?;
    codec1.set_input_volume(AudioChannel::Right, 50)?;

    codec1.set_output_volume(AudioChannel::Left, 0b1100000)?;
    codec1.set_output_volume(AudioChannel::Right, 0b1100000)?;

    codec1.set_mix(AudioChannel::Left, MAX_MIX_VOLUME, 0)?;
    codec1.set_mix(AudioChannel::Right, 0, MAX_MIX_VOLUME)?;

    codec1.set_dac_volume(AudioChannel::Left, MAX_DAC_VOLUME)?;
    codec1.set_dac_volume(AudioChannel::Right, MAX_DAC_VOLUME)?;

    codec1.set_mic_boost(AudioChannel::Left, MicBoost::Db29)?;
    codec1.set_mic_boost(AudioChannel::Right, MicBoost::Db29)?;

    log::info!("Hello, world!");

    Ok(())
}
