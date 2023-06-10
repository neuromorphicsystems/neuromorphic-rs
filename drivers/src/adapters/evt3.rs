#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {}

pub struct Adapter {
    t: u64,
    overflows: u32,
    previous_msb_t: u16,
    previous_lsb_t: u16,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    polarity: neuromorphic_types::DvsPolarity,
}

impl Adapter {
    pub fn from_dimensions(width: u16, height: u16) -> Self {
        Self {
            t: 0,
            overflows: 0,
            previous_msb_t: 0,
            previous_lsb_t: 0,
            x: 0,
            y: 0,
            width,
            height,
            polarity: neuromorphic_types::DvsPolarity::Off,
        }
    }

    pub fn estimate_dvs_events_length(slice: &[u8]) -> usize {
        slice.len() / 2
    }

    pub fn current_t(&self) -> u64 {
        (((self.previous_lsb_t as u32) | ((self.previous_msb_t as u32) << 12)) as u64)
            | ((self.overflows as u64) << 24)
    }

    pub fn convert<HandleDvsEvent, HandleTriggerEvent>(
        &mut self,
        slice: &[u8],
        mut handle_dvs_event: HandleDvsEvent,
        mut handle_trigger_event: HandleTriggerEvent,
    ) where
        HandleDvsEvent: FnMut(neuromorphic_types::DvsEvent<u64, u16, u16>),
        HandleTriggerEvent: FnMut(neuromorphic_types::TriggerEvent<u64, u8>),
    {
        for index in 0..slice.len() / 2 {
            let word = u16::from_le_bytes([slice[index * 2], slice[index * 2 + 1]]);
            match word >> 12 {
                0b0000 => {
                    self.y = word & 0b11111111111;
                    if self.y < self.height {
                        self.y = self.height - 1 - self.y;
                    }
                }
                0b0001 => (),
                0b0010 => {
                    self.x = word & 0b11111111111;
                    self.polarity = if (word & (1 << 11)) > 0 {
                        neuromorphic_types::DvsPolarity::On
                    } else {
                        neuromorphic_types::DvsPolarity::Off
                    };
                    if self.x < self.width && self.y < self.height {
                        handle_dvs_event(neuromorphic_types::DvsEvent {
                            t: self.t,
                            x: self.x,
                            y: self.y,
                            polarity: self.polarity,
                        });
                    }
                }
                0b0011 => {
                    self.x = word & 0b11111111111;
                    self.polarity = if (word & (1 << 11)) > 0 {
                        neuromorphic_types::DvsPolarity::On
                    } else {
                        neuromorphic_types::DvsPolarity::Off
                    };
                }
                0b0100 => {
                    if self.x < self.width && self.y < self.height {
                        for bit in 0..std::cmp::min(12, self.width - self.x) {
                            if (word & (1 << bit)) > 0 {
                                handle_dvs_event(neuromorphic_types::DvsEvent {
                                    t: self.t,
                                    x: self.x,
                                    y: self.y,
                                    polarity: self.polarity,
                                });
                            }
                            self.x += 1;
                        }
                    }
                }
                0b0101 => {
                    if self.x < self.width && self.y < self.height {
                        for bit in 0..std::cmp::min(8, self.width - self.x) {
                            if (word & (1 << bit)) > 0 {
                                handle_dvs_event(neuromorphic_types::DvsEvent {
                                    t: self.t,
                                    x: self.x,
                                    y: self.y,
                                    polarity: self.polarity,
                                });
                            }
                        }
                        self.x += 1;
                    }
                }
                0b0110 => {
                    let lsb_t = word & 0b111111111111;
                    if lsb_t != self.previous_lsb_t {
                        self.previous_lsb_t = lsb_t;
                        let t = (((self.previous_lsb_t as u32)
                            | ((self.previous_msb_t as u32) << 12))
                            as u64)
                            | ((self.overflows as u64) << 24);
                        if t >= self.t {
                            self.t = t;
                        }
                    }
                }
                0b0111 => (),
                0b1000 => {
                    let msb_t = word & 0b111111111111;
                    if msb_t != self.previous_msb_t {
                        if msb_t > self.previous_msb_t {
                            if (msb_t - self.previous_msb_t) < (1 << 11) {
                                self.previous_lsb_t = 0;
                                self.previous_msb_t = msb_t;
                            }
                        } else {
                            if (self.previous_msb_t - msb_t) > (1 << 11) {
                                self.overflows += 1;
                                self.previous_lsb_t = 0;
                                self.previous_msb_t = msb_t;
                            }
                        }
                        let t = (((self.previous_lsb_t as u32)
                            | ((self.previous_msb_t as u32) << 12))
                            as u64)
                            | ((self.overflows as u64) << 24);
                        if t >= self.t {
                            self.t = t;
                        }
                    }
                }
                0b1001 => (),
                0b1010 => handle_trigger_event(neuromorphic_types::TriggerEvent {
                    t: self.t,
                    id: ((word & 0b1111) >> 8) as u8,
                    polarity: if (word & 1) > 0 {
                        neuromorphic_types::TriggerPolarity::Rising
                    } else {
                        neuromorphic_types::TriggerPolarity::Falling
                    },
                }),
                0b1011 | 0b1100 | 0b1101 | 0b1110 | 0b1111 => (),
                _ => (),
            }
        }
    }

    pub fn consume(&mut self, slice: &[u8]) {
        for index in 0..slice.len() / 2 {
            let word = u16::from_le_bytes([slice[index * 2], slice[index * 2 + 1]]);
            match word >> 12 {
                0b0110 => {
                    let lsb_t = word & 0b111111111111;
                    if lsb_t != self.previous_lsb_t {
                        self.previous_lsb_t = lsb_t;
                        let t = (((self.previous_lsb_t as u32)
                            | ((self.previous_msb_t as u32) << 12))
                            as u64)
                            | ((self.overflows as u64) << 24);
                        if t >= self.t {
                            self.t = t;
                        }
                    }
                }
                0b1000 => {
                    let msb_t = word & 0b111111111111;
                    if msb_t != self.previous_msb_t {
                        if msb_t > self.previous_msb_t {
                            if (msb_t - self.previous_msb_t) < (1 << 11) {
                                self.previous_lsb_t = 0;
                                self.previous_msb_t = msb_t;
                            }
                        } else {
                            if (self.previous_msb_t - msb_t) > (1 << 11) {
                                self.overflows += 1;
                                self.previous_lsb_t = 0;
                                self.previous_msb_t = msb_t;
                            }
                        }
                        let t = (((self.previous_lsb_t as u32)
                            | ((self.previous_msb_t as u32) << 12))
                            as u64)
                            | ((self.overflows as u64) << 24);
                        if t >= self.t {
                            self.t = t;
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
