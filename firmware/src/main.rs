use esp_idf_hal::{
    i2s::{
        config::{
            Config as ChannelConfig, DataBitWidth, SlotMode, StdClkConfig, StdConfig,
            StdGpioConfig, StdSlotConfig,
        },
        I2sDriver,
    },
    prelude::Peripherals,
    spi::{config::DriverConfig, SpiDriver},
};

use esp_idf_svc::

use crate::codec::{
    spi::consts::{MicBoost, MAX_DAC_VOLUME, MAX_MIX_VOLUME},
    AudioChannel, Codec, PowerConfig,
};

mod codec;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let spi_driver = SpiDriver::new(
        peripherals.spi2,
        peripherals.pins.gpio32,
        peripherals.pins.gpio22,
        Option::<esp_idf_hal::gpio::AnyIOPin>::None,
        &DriverConfig::new(),
    )?;

    let codec1_i2s = I2sDriver::new_std_bidir(
        peripherals.i2s0,
        &StdConfig::new(
            ChannelConfig::new(),
            StdClkConfig::from_sample_rate_hz(44100),
            StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, SlotMode::Stereo),
            StdGpioConfig::new(false, false, false),
        ),
        peripherals.pins.gpio16,
        peripherals.pins.gpio15,
        peripherals.pins.gpio17,
        Some(peripherals.pins.gpio0),
        peripherals.pins.gpio5,
    )?;

    let mut codec1 = Codec::new(&spi_driver, peripherals.pins.gpio4, codec1_i2s)?;

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

    codec1.set_dac_mute(false)?;

    codec1.i2s_driver.tx_enable()?;

    log::info!("Hello, world!");

    // Write the precomputed wavetable repeatedly — no per-loop computation
    loop {
        // Generate A4 note (440 Hz) samples at 44100 Hz sample rate
        let samples_per_cycle = 44100 / 440; // ~100 samples per cycle
        let mut sample_buffer = vec![0i16; samples_per_cycle * 2]; // stereo, 16-bit samples

        // Fill buffer with sine wave for A4 (440 Hz)
        for i in 0..samples_per_cycle {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / samples_per_cycle as f32;
            let sample = (angle.sin() * 16384.0) as i16; // Scale to ~half of i16 range
            sample_buffer[i * 2] = sample; // Left channel
            sample_buffer[i * 2 + 1] = sample; // Right channel
        }

        // Convert to bytes for I2S transmission
        let byte_buffer: Vec<u8> = sample_buffer
            .iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect();

        // Transmit the audio data
        codec1.i2s_driver.write(&byte_buffer, 1000)?;
    }
}
