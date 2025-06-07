#[cfg(feature = "rand")]
use std::time::{Duration, SystemTime};

/// Represents a Snowflake identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Snowflake(u64);

impl Snowflake {
    /// Returns the underlying `u64` representation of the Snowflake.
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    #[cfg(feature = "rand")]
    fn timestamp_part() -> u64 {
        const EPOCH: Duration = Duration::from_millis(1_546_300_800_000);

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("epoch should be before now");

        let timestamp = timestamp - EPOCH;
        let timestamp = u64::try_from(timestamp.as_millis()).expect("timestamp overflow");

        // The most significant bit (what would be the sign bit) is always
        // 0. The timestamp part then covers the 41 next bits.
        (timestamp & 0x01FF_FFFF_FFFF) << 22
    }

    #[cfg(feature = "rand")]
    /// Generates a new random Snowflake.
    #[must_use]
    pub fn random() -> Self {
        use rand::prelude::*;

        // Random part if the 21 least significant bits of the Snowflake.
        // The 22nd bit is always 0 when generating a random Snowflake.
        let random_part = rand::rng().random_range(0..=0x001F_FFFF);

        Snowflake(Self::timestamp_part() | random_part)
    }
}

/// Creates a [`Snowflake`] from a `u64`.
impl From<u64> for Snowflake {
    fn from(value: u64) -> Self {
        Snowflake(value)
    }
}

#[cfg(feature = "rand")]
/// Provides a default [`Snowflake`] by generating a random one.
///
/// This is equivalent to calling [`Snowflake::random()`].
impl Default for Snowflake {
    fn default() -> Self {
        Self::random()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "rand")]
    #[test]
    fn test_random_snowflake_properties() {
        // Generate a few snowflakes to increase the chance of catching issues
        // with the random part.
        for _ in 0..100 {
            let sf = Snowflake::random();
            let val = sf.as_u64();

            // 1. The most significant bit (bit 63) must be 0.
            assert_eq!((val >> 63) & 1, 0, "Bit 63 (MSB) should be 0. Value: {val:064b}");

            // 2. The 22nd bit (bit 21, 0-indexed) must be 0.
            assert_eq!((val >> 21) & 1, 0, "Bit 21 (22nd bit) should be 0. Value: {val:064b}");

            // 3. The timestamp part (bits 22-62) should generally be non-zero if
            //    time has passed since EPOCH. Mask for bits 22 to 62.
            let timestamp_mask = ((1u64 << 41) - 1) << 22;
            assert_ne!(
                val & timestamp_mask,
                0,
                "Timestamp portion (bits 22-62) should generally be non-zero. Value: {:064b}",
                val & timestamp_mask
            );
        }
    }
}
