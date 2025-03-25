use bitflags::bitflags;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

bitflags! {
    /// Fine-grained permissions built using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FinePermission: u32 {
        const DEFAULT = 1 << 0;
        const ADMINISTRATOR = 1 << 1;
        const MANAGE_JOURNAL_PAGES = 1 << 2;
        const MANAGE_JOURNAL_ENTRIES = 1 << 3;
        const MANAGE_JOURNAL_ENTRY_COMMENTS = 1 << 4;
        const MANAGE_USERS = 1 << 5;
        const MANAGE_BANS = 1 << 6; // includes managing IP bans
        const MANAGE_WARNINGS = 1 << 7;
        const MANAGE_NOTIFICATIONS = 1 << 8;
        const VIEW_REPORTS = 1 << 9;
        const VIEW_AUDIT_LOG = 1 << 10;
        const MANAGE_MEMBERSHIPS = 1 << 11;

        const _ = !0;
    }
}

impl Serialize for FinePermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

struct FinePermissionVisitor;
impl<'de> Visitor<'de> for FinePermissionVisitor {
    type Value = FinePermission;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("u32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = FinePermission::from_bits(value) {
            Ok(permission)
        } else {
            Ok(FinePermission::from_bits_retain(value))
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = FinePermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(FinePermission::from_bits_retain(value as u32))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = FinePermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(FinePermission::from_bits_retain(value as u32))
        }
    }
}

impl<'de> Deserialize<'de> for FinePermission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FinePermissionVisitor)
    }
}

impl FinePermission {
    /// Join two [`FinePermission`]s into a single `u32`.
    pub fn join(lhs: FinePermission, rhs: FinePermission) -> FinePermission {
        lhs | rhs
    }

    /// Check if the given `input` contains the given [`FinePermission`].
    pub fn check(self, permission: FinePermission) -> bool {
        if (self & FinePermission::ADMINISTRATOR) == FinePermission::ADMINISTRATOR {
            // has administrator permission, meaning everything else is automatically true
            return true;
        }

        (self & permission) == permission
    }

    /// Check if the given [`FinePermission`] qualifies as "Helper" status.
    pub fn check_helper(self) -> bool {
        self.check(FinePermission::MANAGE_JOURNAL_ENTRIES)
            && self.check(FinePermission::MANAGE_JOURNAL_PAGES)
            && self.check(FinePermission::MANAGE_JOURNAL_ENTRY_COMMENTS)
            && self.check(FinePermission::MANAGE_WARNINGS)
            && self.check(FinePermission::VIEW_REPORTS)
            && self.check(FinePermission::VIEW_AUDIT_LOG)
    }

    /// Check if the given [`FinePermission`] qualifies as "Manager" status.
    pub fn check_manager(self) -> bool {
        self.check_helper() && self.check(FinePermission::ADMINISTRATOR)
    }
}

impl Default for FinePermission {
    fn default() -> Self {
        Self::DEFAULT
    }
}
