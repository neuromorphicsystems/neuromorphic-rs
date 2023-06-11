#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum DvsPolarity {
    Off = 0,
    On = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct DvsEvent<Timestamp, X, Y> {
    pub t: Timestamp,
    pub x: X,
    pub y: Y,
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

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct AtisEvent<Timestamp, X, Y> {
    pub t: Timestamp,
    pub x: X,
    pub y: Y,
    pub polarity: AtisPolarity,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum TriggerPolarity {
    Falling = 0,
    Rising = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct TriggerEvent<Timestamp, Id> {
    pub t: Timestamp,
    pub id: Id,
    pub polarity: TriggerPolarity,
}
