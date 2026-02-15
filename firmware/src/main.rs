use esp_idf_hal::{
    delay::FreeRtos,
    i2s::{
        config::{
            Config as ChannelConfig, DataBitWidth, SlotMode, StdClkConfig, StdConfig,
            StdGpioConfig, StdSlotConfig,
        },
        I2sDriver,
    },
    prelude::*,
    spi::{config::DriverConfig, SpiDriver},
};
use esp_idf_svc::{
    bt::{
        a2dp::{A2dpEvent, ConnectionStatus, EspA2dp},
        gap::{EspGap, GapEvent},
        BtClassic, BtDriver, BtStatus,
    },
    nvs::EspDefaultNvsPartition,
};

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

    codec1.set_dac_mute(true)?;

    codec1.i2s_driver.tx_enable()?;

    let nvs = EspDefaultNvsPartition::take()?;

    let driver = BtDriver::<BtClassic>::new(peripherals.modem, Some(nvs))?;

    driver.set_device_name("CosplayCore")?;

    let gap_server = EspGap::new(&driver)?;

    gap_server.set_scan_mode(true, esp_idf_svc::bt::gap::DiscoveryMode::Discoverable)?;

    gap_server.subscribe(move |event| match event {
        GapEvent::AuthenticationCompleted {
            status,
            device_name,
            ..
        } => {
            if status == BtStatus::Success {
                log::info!("Bluetooth authentication success: {device_name}");
            } else {
                log::error!("Bluetooth authentication failed, status: {status:?}");
            }
        }
        _ => {}
    })?;

    let a2dp_sink = EspA2dp::new_sink(&driver)?;

    // TODO: Make this not bad
    unsafe {
        a2dp_sink.subscribe_nonstatic(move |event| {
            match event {
                A2dpEvent::SinkData(data) => {
                    codec1.i2s_driver.write(data, 1000).unwrap();
                }
                A2dpEvent::ConnectionState { status, .. } => {
                    if status == ConnectionStatus::Connected {
                        log::info!("Bluetooth device connected");

                        codec1.set_dac_mute(false).unwrap();
                    } else {
                        codec1.set_dac_mute(true).unwrap();
                    }
                }
                A2dpEvent::AudioCodecConfigured { codec, .. } => {
                    log::info!("Connected to codec: {codec:?}");
                    // TODO: Use an audio resampler so I2S can stick to 44100Hz
                }
                _ => {}
            };
            0
        })?;
    }
    log::info!("Hello, world!");

    loop {
        FreeRtos::delay_ms(20);
    }
}
