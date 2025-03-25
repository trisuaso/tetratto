use bitflags::bitflags;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

bitflags! {
    /// Fine-grained journal permissions built using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct JournalPermission: u32 {
        const DEFAULT = 1 << 0;
        const ADMINISTRATOR = 1 << 1;
        const MEMBER = 1 << 2;

        const _ = !0;
    }
}

impl Serialize for JournalPermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

struct JournalPermissionVisitor;
impl<'de> Visitor<'de> for JournalPermissionVisitor {
    type Value = JournalPermission;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("u32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = JournalPermission::from_bits(value) {
            Ok(permission)
        } else {
            Ok(JournalPermission::from_bits_retain(value))
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = JournalPermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(JournalPermission::from_bits_retain(value as u32))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = JournalPermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(JournalPermission::from_bits_retain(value as u32))
        }
    }
}

impl<'de> Deserialize<'de> for JournalPermission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JournalPermissionVisitor)
    }
}

impl JournalPermission {
    /// Join two [`JournalPermission`]s into a single `u32`.
    pub fn join(lhs: JournalPermission, rhs: JournalPermission) -> JournalPermission {
        lhs | rhs
    }

    /// Check if the given `input` contains the given [`JournalPermission`].
    pub fn check(self, permission: JournalPermission) -> bool {
        if (self & JournalPermission::ADMINISTRATOR) == JournalPermission::ADMINISTRATOR {
            // has administrator permission, meaning everything else is automatically true
            return true;
        }

        (self & permission) == permission
    }

    /// Check if the given [`JournalPermission`] qualifies as "Member" status.
    pub fn check_helper(self) -> bool {
        self.check(JournalPermission::MEMBER)
    }
}

impl Default for JournalPermission {
    fn default() -> Self {
        Self::DEFAULT
    }
}
