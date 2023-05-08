use std::cmp::Ordering;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct S4Vector {
    pub ssn: u32, // session number unused in this implementation
    pub sid: u32,
    pub sum: u32,
    pub seq: u32,
}

impl S4Vector {
    pub fn root() -> S4Vector {
        S4Vector {
            ssn: 0,
            sid: 0,
            sum: 0,
            seq: 0,
        }
    }

    pub fn to_array(&self) -> [u32; 4] {
        [self.ssn, self.sid, self.sum, self.seq]
    }
}

impl Ord for S4Vector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.ssn < other.ssn {
            Ordering::Less
        } else if self.ssn > other.ssn {
            Ordering::Greater
        } else if self.sum < other.sum {
            Ordering::Less
        } else if self.sum > other.sum {
            Ordering::Greater
        } else if self.sid < other.sid {
            Ordering::Less
        } else if self.sid > other.sid {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for S4Vector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<[u32; 4]> for S4Vector {
    fn from(value: [u32; 4]) -> Self {
        S4Vector {
            ssn: value[0],
            sid: value[1],
            sum: value[2],
            seq: value[3],
        }
    }
}

pub struct VectorClock {
    clock: Vec<u32>,
    site_id: usize,
}

impl VectorClock {
    pub fn new(site_id: usize) -> VectorClock {
        VectorClock {
            site_id,
            clock: vec![0; site_id + 1],
        }
    }

    pub fn from_parts(site_id: usize, clock: Vec<u32>) -> VectorClock {
        VectorClock { site_id, clock }
    }

    pub fn id(&self) -> usize {
        self.site_id
    }

    pub fn increase(&mut self) {
        self.clock[self.site_id] += 1;
    }

    pub fn merge_remote(&mut self, values: &[u32]) {
        for (clock_value, new_value) in self.clock.iter_mut().zip(values) {
            *clock_value = (*clock_value).max(*new_value)
        }
        if values.len() > self.clock.len() {
            self.clock.extend_from_slice(&values[self.clock.len()..]);
        }
    }
    pub fn to_s4vector(&self) -> S4Vector {
        let sum = self.clock.iter().sum();
        S4Vector {
            ssn: 0,
            sid: self.site_id as u32,
            sum,
            seq: self.clock[self.site_id] as u32,
        }
    }

    pub fn clock_value(&self, s_id: usize) -> u32 {
        self.clock.get(s_id).copied().unwrap_or(0)
    }

    pub fn clock_values(&self) -> &[u32] {
        &self.clock
    }
}

#[test]
fn test_merge_clocks() {
    let mut vc = VectorClock::new(3);
    vc.merge_remote(&[2]);
    vc.merge_remote(&[0, 1, 0, 0, 2]);
    assert_eq!(vc.clock.as_slice(), [2, 1, 0, 0, 2])
}
