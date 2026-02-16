use std::sync::Arc;

use esp_idf_hal::{
    gpio::OutputPin,
    i2s::{I2sBiDir, I2sDriver},
    peripheral::Peripheral,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriver},
};

use crate::codec::spi::consts::{MicBoost, MAX_DAC_VOLUME, MAX_MIX_VOLUME};
pub mod spi;

#[derive(PartialEq, Copy, Clone)]
pub enum AudioChannel {
    Left,
    Right,
}

#[derive(Default, Clone, Copy)]
pub struct ChannelPair<T: Default> {
    left: T,
    right: T,
}

impl<T: Default + Copy> ChannelPair<T> {
    pub fn new(left: T, right: T) -> Self {
        Self { left, right }
    }

    pub fn get(&self, channel: AudioChannel) -> T {
        match channel {
            AudioChannel::Left => self.left,
            AudioChannel::Right => self.right,
        }
    }

    pub fn set(&mut self, channel: AudioChannel, value: T) {
        match channel {
            AudioChannel::Left => self.left = value,
            AudioChannel::Right => self.right = value,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PowerConfig {
    pub adc_left: bool,
    pub adc_right: bool,
    pub dac_left: bool,
    pub dac_right: bool,
    pub left_out_1: bool,
    pub right_out_1: bool,
    pub pga_left: bool,
    pub pga_right: bool,
}

pub struct Codec<'a> {
    spi_device: SpiDeviceDriver<'a, Arc<SpiDriver<'a>>>,

    i2s_driver: I2sDriver<'a, I2sBiDir>,

    bluetooth_receiver: &'static global_channel::crossbeam_channel::Receiver<Vec<u8>>,

    input_volume: ChannelPair<u8>,

    output_volume: ChannelPair<u8>,

    mix: ChannelPair<ChannelPair<u8>>,

    power_config: PowerConfig,

    dac_volume: ChannelPair<u8>,
}

impl<'a> Codec<'a> {
    pub fn new(
        spi_driver: Arc<SpiDriver<'a>>,
        spi_cs: impl Peripheral<P = impl OutputPin> + 'a,
        i2s_driver: I2sDriver<'a, I2sBiDir>,
        bluetooth_receiver: &'static global_channel::crossbeam_channel::Receiver<Vec<u8>>,
    ) -> anyhow::Result<Self> {
        let mut this = Self {
            spi_device: SpiDeviceDriver::new(spi_driver, Some(spi_cs), &SpiConfig::new())?,
            i2s_driver,
            bluetooth_receiver,
            input_volume: Default::default(),
            output_volume: Default::default(),
            power_config: Default::default(),
            mix: Default::default(),
            dac_volume: Default::default(),
        };

        this.reset_codec()?;

        Ok(this)
    }

    pub fn entrypoint(mut self) -> anyhow::Result<()> {
        self.set_power_management(PowerConfig {
            adc_left: false,
            adc_right: false,
            dac_left: true,
            dac_right: true,
            left_out_1: true,
            right_out_1: true,
            pga_left: true,
            pga_right: true,
        })?;

        self.set_input_volume(AudioChannel::Left, 50)?;
        self.set_input_volume(AudioChannel::Right, 50)?;

        self.set_output_volume(AudioChannel::Left, 0b1100000)?;
        self.set_output_volume(AudioChannel::Right, 0b1100000)?;

        self.set_mix(AudioChannel::Left, MAX_MIX_VOLUME, 0)?;
        self.set_mix(AudioChannel::Right, 0, MAX_MIX_VOLUME)?;

        self.set_dac_volume(AudioChannel::Left, MAX_DAC_VOLUME)?;
        self.set_dac_volume(AudioChannel::Right, MAX_DAC_VOLUME)?;

        self.set_mic_boost(AudioChannel::Left, MicBoost::Db29)?;
        self.set_mic_boost(AudioChannel::Right, MicBoost::Db29)?;

        self.set_dac_mute(false)?;

        self.i2s_driver.tx_enable()?;

        loop {
            let data = self.bluetooth_receiver.recv()?;

            self.i2s_driver.write_all(data.as_slice(), 1000)?;
        }
    }
}
