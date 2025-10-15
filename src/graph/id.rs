use std::{
    iter::*,
    ops::RangeInclusive,
    sync::{
        LazyLock,
        nonpoison::{Mutex, MutexGuard},
    },
};
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("out of IDs")]
pub struct OutOfIDsError;

type IdIter<T, Idx> = Map<RangeInclusive<Idx>, fn(Idx) -> T>;

#[derive(Debug, Error)]
pub enum ParseIdError<const PREFIX: char> {
    #[error("ID missing prefix: {PREFIX}")]
    MissingPrefix,

    #[error("ID number could not be parsed")]
    ParseInt(std::num::ParseIntError),
}

macro_rules! define_id {
    (#[prefix = $prefix:literal] $vis:vis struct $Id:ident($inner_vis:vis $Repr:ty)) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $Id($inner_vis $Repr);

        impl std::fmt::Display for $Id {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{:x}", $prefix, self.0)
            }
        }

        impl std::str::FromStr for $Id {
            type Err = ParseIdError<$prefix>;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.strip_prefix($prefix) {
                    Some(x) => match <$Repr>::from_str_radix(x, 16) {
                        Ok(n) => Ok(Self(n)),
                        Err(e) => Err(ParseIdError::ParseInt(e)),
                    }
                    None => Err(ParseIdError::MissingPrefix),
                }
            }
        }

        impl $Id {
            #[inline]
            pub fn next() -> Result<Self, OutOfIDsError> {
                Self::iter().next().ok_or(OutOfIDsError)
            }

            #[inline]
            pub fn iter<'a>() -> MutexGuard<'a, IdIter<$Id, $Repr>> {
                static GLOBAL_IDS: LazyLock<Mutex<IdIter<$Id, $Repr>>> =
                    LazyLock::new(|| Mutex::new((0..=!0).map($Id)));
                GLOBAL_IDS.lock()
            }
        }
    };
}

define_id!(#[prefix = 'g'] pub struct GraphId(pub(in crate::graph) u32));
define_id!(#[prefix = 'n'] pub struct NodeId(pub(in crate::graph) u128));
define_id!(#[prefix = 'w'] pub struct WireId(pub(in crate::graph) u128));
