#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum RegisterAddress {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AudioWordLength {
    Bit16 = 0b00,
    Bit20 = 0b01,
    Bit24 = 0b10,
    Bit32 = 0b11,
}

pub const MAX_INPUT_VOLUME: u8 = 0b111111;
pub const MAX_OUTPUT_VOLUME: u8 = 0b1111111;
pub const MAX_MIX_VOLUME: u8 = 0b111;
pub const MAX_DAC_VOLUME: u8 = 0b11111111;
