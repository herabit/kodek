#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Endian {
    Little,
    #[default]
    Big,
}

impl Endian {
    pub const NATIVE: Self = if cfg!(target_endian = "little") {
        Self::Little
    } else {
        Self::Big
    };

    #[inline]
    #[must_use]
    pub const fn is_little(self) -> bool {
        matches!(self, Self::Little)
    }

    #[inline]
    #[must_use]
    pub const fn is_big(self) -> bool {
        matches!(self, Self::Big)
    }

    #[inline]
    #[must_use]
    pub const fn is_native(self) -> bool {
        matches!(self, Self::NATIVE)
    }
}
