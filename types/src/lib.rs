#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum DvsPolarity {
    Off = 0,
    On = 1,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct DvsEvent<Timestamp, Width, Height> {
    pub t: Timestamp,
    pub x: Width,
    pub y: Height,
    pub polarity: DvsPolarity,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum AtisPolarity {
    Off = 0,
    On = 1,
    ExposureStart = 2,
    ExposureEnd = 3,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct AtisEvent<Timestamp, Width, Height> {
    pub t: Timestamp,
    pub x: Width,
    pub y: Height,
    pub polarity: AtisPolarity,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum TriggerPolarity {
    Falling = 0,
    Rising = 1,
}

#[repr(packed)]
#[derive(Debug, Copy, Clone)]
pub struct TriggerEvent<Timestamp, Id> {
    pub t: Timestamp,
    pub system_t: std::time::Instant,
    pub id: Id,
    pub polarity: TriggerPolarity,
}
