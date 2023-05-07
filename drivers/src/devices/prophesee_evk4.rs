use crate::properties;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Biases {
    pub pr: u8,
    pub fo: u8,
    pub hpf: u8,
    pub diff_on: u8,
    pub diff: u8,
    pub diff_off: u8,
    pub inv: u8,
    pub refr: u8,
    pub reqpuy: u8,
    pub reqpux: u8,
    pub sendreqpdy: u8,
    pub unknown_1: u8,
    pub unknown_2: u8,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Configuration {
    pub biases: Biases,
}

pub const PROPERTIES: properties::Camera<Configuration> = properties::Camera::<Configuration> {
    name: "Prophesee EVK4",
    width: 1280,
    height: 720,
    default_configuration: Configuration {
        biases: Biases {
            pr: 0x7C,
            fo: 0x53,
            hpf: 0x00,
            diff_on: 0x66,
            diff: 0x4D,
            diff_off: 0x49,
            inv: 0x5B,
            refr: 0x14,
            reqpuy: 0x8C,
            reqpux: 0x7C,
            sendreqpdy: 0x94,
            unknown_1: 0x74,
            unknown_2: 0x51,
        },
    },
};
