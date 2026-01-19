#[derive(Debug, Clone, PartialEq)]
pub struct Storage {
    pub(crate) quotient: u64,
    pub(crate) remainder: u64,
    pub(crate) unit: Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    B,
    KB,
    MB,
    GB,
    TB,
    PB,
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Unit::B => "B",
                Unit::KB => "KB",
                Unit::MB => "MB",
                Unit::GB => "GB",
                Unit::TB => "TB",
                Unit::PB => "PB",
            }
        )
    }
}

impl Storage {
    const SHIFT_KB: u64 = 10;
    const SHIFT_MB: u64 = 20;
    const SHIFT_GB: u64 = 30;
    const SHIFT_TB: u64 = 40;
    const SHIFT_PB: u64 = 50;

    const SCALE_KB: f64 = 1.0 / (1u64 << Self::SHIFT_KB) as f64;
    const SCALE_MB: f64 = 1.0 / (1u64 << Self::SHIFT_MB) as f64;
    const SCALE_GB: f64 = 1.0 / (1u64 << Self::SHIFT_GB) as f64;
    const SCALE_TB: f64 = 1.0 / (1u64 << Self::SHIFT_TB) as f64;
    const SCALE_PB: f64 = 1.0 / (1u64 << Self::SHIFT_PB) as f64;

    pub fn new(quotient: u64, remainder: u64, unit: Unit) -> Self {
        Self {
            quotient,
            remainder,
            unit,
        }
    }

    pub fn from_bytes(bytes: u64) -> Self {
        if bytes >= (1 << Self::SHIFT_PB) {
            let q = bytes >> Self::SHIFT_PB;
            let r = bytes & ((1 << Self::SHIFT_PB) - 1);
            Storage::new(q, r, Unit::PB)
        } else if bytes >= (1 << Self::SHIFT_TB) {
            let q = bytes >> Self::SHIFT_TB;
            let r = bytes & ((1 << Self::SHIFT_TB) - 1);
            Storage::new(q, r, Unit::TB)
        } else if bytes >= (1 << Self::SHIFT_GB) {
            let q = bytes >> Self::SHIFT_GB;
            let r = bytes & ((1 << Self::SHIFT_GB) - 1);
            Storage::new(q, r, Unit::GB)
        } else if bytes >= (1 << Self::SHIFT_MB) {
            let q = bytes >> Self::SHIFT_MB;
            let r = bytes & ((1 << Self::SHIFT_MB) - 1);
            Storage::new(q, r, Unit::MB)
        } else if bytes >= (1 << Self::SHIFT_KB) {
            let q = bytes >> Self::SHIFT_KB;
            let r = bytes & ((1 << Self::SHIFT_KB) - 1);
            Storage::new(q, r, Unit::KB)
        } else {
            Storage::new(bytes, 0, Unit::B)
        }
    }

    pub fn to_bytes(&self) -> u64 {
        match self.unit {
            Unit::B => self.quotient,
            Unit::KB => (self.quotient << Self::SHIFT_KB) | self.remainder,
            Unit::MB => (self.quotient << Self::SHIFT_MB) | self.remainder,
            Unit::GB => (self.quotient << Self::SHIFT_GB) | self.remainder,
            Unit::TB => (self.quotient << Self::SHIFT_TB) | self.remainder,
            Unit::PB => (self.quotient << Self::SHIFT_PB) | self.remainder,
        }
    }

    pub fn to_float(&self) -> f64 {
        match self.unit {
            Unit::B => self.quotient as f64,
            Unit::KB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_KB),
            Unit::MB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_MB),
            Unit::GB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_GB),
            Unit::TB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_TB),
            Unit::PB => self.quotient as f64 + (self.remainder as f64 * Self::SCALE_PB),
        }
    }

    pub fn quotient(&self) -> u64 {
        self.quotient
    }

    pub fn remainder(&self) -> u64 {
        self.remainder
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }
}

impl std::ops::Add<Storage> for Storage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let total_bytes = self.to_bytes() + other.to_bytes();
        Storage::from_bytes(total_bytes)
    }
}

impl std::ops::Add<&Storage> for Storage {
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        let total_bytes = self.to_bytes() + other.to_bytes();
        Storage::from_bytes(total_bytes)
    }
}

impl std::ops::Add<Storage> for &Storage {
    type Output = Storage;

    fn add(self, other: Storage) -> Self::Output {
        let total_bytes = self.to_bytes() + other.to_bytes();
        Storage::from_bytes(total_bytes)
    }
}

impl std::ops::Add<&Storage> for &Storage {
    type Output = Storage;

    fn add(self, other: &Storage) -> Self::Output {
        let total_bytes = self.to_bytes() + other.to_bytes();
        Storage::from_bytes(total_bytes)
    }
}

impl std::fmt::Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.to_float();
        write!(f, "{:.2} {}", value, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KB: u64 = 1 << 10;
    const MB: u64 = 1 << 20;
    const GB: u64 = 1 << 30;
    const TB: u64 = 1 << 40;
    const PB: u64 = 1 << 50;

    #[test]
    fn test_from_bytes() {
        // (bytes, quotient, remainder, unit)
        let cases: &[(u64, u64, u64, Unit)] = &[
            (0, 0, 0, Unit::B),
            (1, 1, 0, Unit::B),
            (512, 512, 0, Unit::B),
            (1023, 1023, 0, Unit::B),
            (KB, 1, 0, Unit::KB),
            (MB, 1, 0, Unit::MB),
            (GB, 1, 0, Unit::GB),
            (TB, 1, 0, Unit::TB),
            (PB, 1, 0, Unit::PB),
            (1536, 1, 512, Unit::KB),
            (MB + 512, 1, 512, Unit::MB),
            (3 * MB + 256, 3, 256, Unit::MB),
            (5 * GB + MB, 5, MB, Unit::GB),
            (MB - 1, 1023, 1023, Unit::KB),
            (10 * PB, 10, 0, Unit::PB),
            (100 * PB + 512, 100, 512, Unit::PB),
        ];
        for &(bytes, q, r, unit) in cases {
            assert_eq!(
                Storage::from_bytes(bytes),
                Storage::new(q, r, unit),
                "from_bytes({bytes})"
            );
        }
    }

    #[test]
    fn test_to_bytes() {
        // (quotient, remainder, unit, expected_bytes)
        let cases: &[(u64, u64, Unit, u64)] = &[
            (0, 0, Unit::B, 0),
            (1, 0, Unit::B, 1),
            (1023, 0, Unit::B, 1023),
            (1, 0, Unit::KB, KB),
            (1, 0, Unit::MB, MB),
            (1, 0, Unit::GB, GB),
            (1, 0, Unit::TB, TB),
            (1, 0, Unit::PB, PB),
            (1, 512, Unit::KB, 1536),
            (1, 512, Unit::MB, MB + 512),
            (3, 256, Unit::MB, 3 * MB + 256),
            (5, MB, Unit::GB, 5 * GB + MB),
            (100, 512, Unit::PB, 100 * PB + 512),
            (1023, 1023, Unit::KB, 1023 * KB + 1023),
            (10, 0, Unit::TB, 10 * TB),
        ];
        for &(q, r, unit, expected) in cases {
            assert_eq!(Storage::new(q, r, unit).to_bytes(), expected);
        }
    }

    #[test]
    fn test_round_trip() {
        let cases = [
            0,
            1,
            512,
            1023,
            KB,
            KB + 1,
            1536,
            MB - 1,
            MB,
            MB + 512,
            3 * MB + 256,
            GB - 1,
            GB,
            5 * GB + MB,
            TB,
            10 * PB,
            100 * PB + 512,
        ];
        for bytes in cases {
            assert_eq!(
                Storage::from_bytes(bytes).to_bytes(),
                bytes,
                "round trip {bytes}"
            );
        }
    }

    #[test]
    fn test_boundary_values() {
        // (bytes, quotient, remainder, unit)
        let cases: &[(u64, u64, u64, Unit)] = &[
            (KB - 1, 1023, 0, Unit::B),
            (KB, 1, 0, Unit::KB),
            (KB + 1, 1, 1, Unit::KB),
            (MB - 1, 1023, 1023, Unit::KB),
            (MB, 1, 0, Unit::MB),
            (MB + 1, 1, 1, Unit::MB),
            (GB - 1, 1023, MB - 1, Unit::MB),
            (GB, 1, 0, Unit::GB),
            (GB + 1, 1, 1, Unit::GB),
            (TB, 1, 0, Unit::TB),
            (PB, 1, 0, Unit::PB),
        ];
        for &(bytes, q, r, unit) in cases {
            assert_eq!(
                Storage::from_bytes(bytes),
                Storage::new(q, r, unit),
                "boundary {bytes}"
            );
        }
    }

    #[test]
    fn test_to_float() {
        const E: f64 = 1e-10;
        // (quotient, remainder, unit, expected_float)
        let cases: &[(u64, u64, Unit, f64)] = &[
            (0, 0, Unit::B, 0.0),
            (1, 0, Unit::B, 1.0),
            (512, 0, Unit::B, 512.0),
            (1023, 0, Unit::B, 1023.0),
            (1, 0, Unit::KB, 1.0),
            (5, 0, Unit::KB, 5.0),
            (1, 512, Unit::KB, 1.5),
            (2, 256, Unit::KB, 2.25),
            (1, 0, Unit::MB, 1.0),
            (3, 0, Unit::MB, 3.0),
            (1, 0, Unit::GB, 1.0),
            (5, 0, Unit::GB, 5.0),
            (1, 0, Unit::TB, 1.0),
            (10, 0, Unit::TB, 10.0),
            (1, 0, Unit::PB, 1.0),
            (100, 0, Unit::PB, 100.0),
            (1000, 0, Unit::PB, 1000.0),
            (1023, 1023, Unit::KB, 1023.0 + 1023.0 / 1024.0),
        ];
        for &(q, r, unit, expected) in cases {
            let v = Storage::new(q, r, unit).to_float();
            assert!(
                (v - expected).abs() < E,
                "to_float({q},{r},{unit:?}): got {v}, want {expected}"
            );
        }
    }

    #[test]
    fn test_to_float_round_trip() {
        const E: f64 = 1e-5;
        let cases = [
            (0, 0, Unit::B),
            (1, 0, Unit::B),
            (512, 0, Unit::B),
            (1023, 0, Unit::B),
            (1, 0, Unit::KB),
            (1, 512, Unit::KB),
            (2, 256, Unit::KB),
            (1, 0, Unit::MB),
            (1, 512, Unit::MB),
            (3, 256, Unit::MB),
            (1, 0, Unit::GB),
            (5, MB, Unit::GB),
            (1, 0, Unit::TB),
            (2, 512, Unit::TB),
            (1, 0, Unit::PB),
            (100, 512, Unit::PB),
        ];
        for (q, r, unit) in cases {
            let s = Storage::new(q, r, unit);
            let bytes = s.to_bytes();
            let scale = match unit {
                Unit::B => 1.0,
                Unit::KB => KB as f64,
                Unit::MB => MB as f64,
                Unit::GB => GB as f64,
                Unit::TB => TB as f64,
                Unit::PB => PB as f64,
            };
            let calc = s.to_float() * scale;
            let diff = (calc - bytes as f64).abs();
            assert!(
                diff < E * bytes as f64 || diff < 1.0,
                "float round trip {s:?}"
            );
        }
    }
}
