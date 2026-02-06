use core::{cell::RefCell, convert::Into, ops::Range};

use embedded_hal::{
    digital::OutputPin,
    spi::{SpiBus, SpiDevice},
};
use embedded_hal_bus::spi::{DeviceError, RefCellDevice};
use esp_hal::delay::Delay;

#[derive(PartialEq, Copy, Clone)]
pub enum AudioChannel {
    Left,
    Right,
}

#[derive(Default, Clone, Copy)]
struct ChannelPair<T: Default> {
    left: T,
    right: T,
}

impl<T: Default + Copy> ChannelPair<T> {
    pub fn new(left: T, right: T) -> Self {
        Self { left, right }
    }

    fn get(&self, channel: AudioChannel) -> T {
        match channel {
            AudioChannel::Left => self.left,
            AudioChannel::Right => self.right,
        }
    }

    fn set(&mut self, channel: AudioChannel, value: T) {
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

pub struct Codec<'a, BUS: SpiBus, CS: OutputPin> {
    device: RefCellDevice<'a, BUS, CS, Delay>,

    input_volume: ChannelPair<u8>,

    output_volume: ChannelPair<u8>,

    mix: ChannelPair<ChannelPair<u8>>,

    power_config: PowerConfig,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RegisterAddress {
    LeftInputVolume = 0x00,
    RightInputVolume = 0x01,
    LeftOutput1Volume = 0x02,
    RightOutput1Volume = 0x03,
    ADCDACControl = 0x05,
    AudioInterface = 0x07,
    SampleRate = 0x08,
    LeftDACVolume = 0x0A,
    RightDACVolume = 0x0B,
    BassControl = 0x0C,
    TrebleControl = 0x0D,
    Reset = 0x0F,
    Control3D = 0x10,
    ALC1 = 0x11,
    ALC2 = 0x12,
    ALC3 = 0x13,
    NoiseGate = 0x14,
    LeftADCVolume = 0x15,
    RightADCVolume = 0x16,
    AdditionalControl1 = 0x17,
    AdditionalControl2 = 0x18,
    PowerManagement1 = 0x19,
    PowerManagement2 = 0x1A,
    AdditionalControl3 = 0x1B,
    ADCInputMode = 0x1F,
    ADCLSignalPath = 0x20,
    ADCRSignalPath = 0x21,
    LeftOutMix1 = 0x22,
    LeftOutMix2 = 0x23,
    RightOutMix1 = 0x24,
    RightOutMix2 = 0x25,
    LeftOutput2Volume = 0x28,
    RightOutput2Volume = 0x29,
    LowPowerPlayback = 0x43,
}

fn mask(n: u8) -> u16 {
    if n >= 16 {
        u16::MAX
    } else if n == 0 {
        0
    } else {
        (1u16 << n) - 1
    }
}

fn pack<T: Into<u16>>(value: T, bits: Range<u8>) -> u16 {
    let repr: u16 = value.into();
    let n = bits.end - bits.start;

    let max = 2_u16.pow(n as u32) - 1;

    if repr > max {
        panic!("Attempted to mask integer with value {repr} to {n} bits, whose maximum is {max}");
    }

    return (repr & mask(n)) << bits.start;
}

pub const MAX_INPUT_VOLUME: u8 = 0b111111;
pub const MAX_OUTPUT_VOLUME: u8 = 0b1111111;
pub const MAX_MIX_VOLUME: u8 = 0b111;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MixInputSelection {
    Input1 = 0b000,
    Input2 = 0b001,
    MicBoost = 0b011,
    Differential = 0b100,
}

impl<'a, BUS: SpiBus, CS: OutputPin> Codec<'a, BUS, CS> {
    pub fn new(bus: &'a RefCell<BUS>, cs: CS) -> Result<Self, DeviceError<BUS::Error, CS::Error>> {
        let mut this = Self {
            device: RefCellDevice::new(bus, cs, Delay::new())
                .map_err(|err| DeviceError::<BUS::Error, CS::Error>::Cs(err))?,
            input_volume: Default::default(),
            output_volume: Default::default(),
            power_config: Default::default(),
            mix: Default::default(),
        };

        this.reset_codec()?;

        Ok(this)
    }

    fn write_register(
        &mut self,
        address: RegisterAddress,
        value: u16,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let data = pack(address as u16, 9..16) | pack(value, 0..9);

        self.device.write(&data.to_be_bytes())
    }

    pub fn get_input_volume(&self, channel: AudioChannel) -> u8 {
        self.input_volume.get(channel)
    }

    pub fn set_input_volume(
        &mut self,
        channel: AudioChannel,
        volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address = match channel {
            AudioChannel::Left => RegisterAddress::LeftInputVolume,
            AudioChannel::Right => RegisterAddress::RightInputVolume,
        };

        let update_immediate = true;
        let mute = volume == 0;
        let zero_cross_detector = false;

        let data = pack(update_immediate, 8..9)
            | pack(mute, 7..8)
            | pack(zero_cross_detector, 6..7)
            | pack(volume, 0..6);

        self.write_register(address, data)?;

        self.input_volume.set(channel, volume);

        Ok(())
    }

    pub fn get_output_volume(&self, channel: AudioChannel) -> u8 {
        self.output_volume.get(channel)
    }

    pub fn set_output_volume(
        &mut self,
        channel: AudioChannel,
        volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address = match channel {
            AudioChannel::Left => RegisterAddress::LeftOutput1Volume,
            AudioChannel::Right => RegisterAddress::RightOutput1Volume,
        };

        let update_immediate = true;

        self.write_register(address, pack(update_immediate, 8..9) | pack(volume, 0..8))?;

        self.output_volume.set(channel, volume);

        Ok(())
    }

    pub fn get_power_config(&self) -> PowerConfig {
        self.power_config
    }

    pub fn set_power_management(
        &mut self,
        config: PowerConfig,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let vmid_selection: u8 = 0b01;
        let vref = true;

        let lout2 = false;
        let rout2 = false;

        let master_clk_disabled = false;

        let data1 = pack(vmid_selection, 7..9)
            | pack(vref, 6..7)
            | pack(config.pga_left, 5..6)
            | pack(config.pga_right, 4..5)
            | pack(config.adc_left, 3..4)
            | pack(config.adc_right, 2..3)
            | pack(master_clk_disabled, 0..1);

        let data2 = pack(config.dac_left, 8..9)
            | pack(config.dac_right, 7..8)
            | pack(config.left_out_1, 6..7)
            | pack(config.right_out_1, 5..6)
            | pack(lout2, 4..5)
            | pack(rout2, 3..4);

        self.write_register(RegisterAddress::PowerManagement1, data1)?;

        self.write_register(RegisterAddress::PowerManagement2, data2)?;

        self.power_config = config;

        Ok(())
    }

    pub fn get_mix(&self, channel: AudioChannel) -> ChannelPair<u8> {
        self.mix.get(channel)
    }

    pub fn set_mix(
        &mut self,
        channel: AudioChannel,
        left_volume: u8,
        right_volume: u8,
    ) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        let address1 = if channel == AudioChannel::Left {
            RegisterAddress::LeftOutMix1
        } else {
            RegisterAddress::RightOutMix1
        };
        let address2 = if channel == AudioChannel::Left {
            RegisterAddress::LeftOutMix2
        } else {
            RegisterAddress::RightOutMix2
        };

        let mix_input_selection = MixInputSelection::MicBoost;

        let left_dac = channel == AudioChannel::Left;
        let right_dac = channel == AudioChannel::Right;

        let left_mix_enabled = left_volume != 0;
        let right_mix_enabled = right_volume != 0;

        let data1 = pack(left_dac, 8..9)
            | pack(left_mix_enabled, 7..8)
            | pack(MAX_MIX_VOLUME - left_volume, 4..7)
            | pack(mix_input_selection as u8, 0..3);

        let data2 = pack(right_dac, 8..9)
            | pack(right_mix_enabled, 7..8)
            | pack(MAX_MIX_VOLUME - right_volume, 4..7);

        self.write_register(address1, data1)?;
        self.write_register(address2, data2)?;

        self.mix
            .set(channel, ChannelPair::new(left_volume, right_volume));

        Ok(())
    }

    pub fn reset_codec(&mut self) -> Result<(), DeviceError<BUS::Error, CS::Error>> {
        self.write_register(RegisterAddress::Reset, 0)?;

        self.set_input_volume(AudioChannel::Left, 0)?;
        self.set_input_volume(AudioChannel::Right, 0)?;

        self.set_output_volume(AudioChannel::Left, 0)?;
        self.set_output_volume(AudioChannel::Right, 0)?;

        self.set_power_management(Default::default())?;

        self.set_mix(AudioChannel::Left, 0, 0)?;
        self.set_mix(AudioChannel::Right, 0, 0)?;

        Ok(())
    }
}
