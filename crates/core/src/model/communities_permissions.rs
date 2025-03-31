use bitflags::bitflags;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as DeError, Visitor},
};

bitflags! {
    /// Fine-grained journal permissions built using bitwise operations.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CommunityPermission: u32 {
        const DEFAULT = 1 << 0;
        const ADMINISTRATOR = 1 << 1;
        const MEMBER = 1 << 2;
        const MANAGE_POSTS = 1 << 3;
        const MANAGE_ROLES = 1 << 4;
        const BANNED = 1 << 5;

        const _ = !0;
    }
}

impl Serialize for CommunityPermission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

struct JournalPermissionVisitor;
impl<'de> Visitor<'de> for JournalPermissionVisitor {
    type Value = CommunityPermission;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("u32")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = CommunityPermission::from_bits(value) {
            Ok(permission)
        } else {
            Ok(CommunityPermission::from_bits_retain(value))
        }
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = CommunityPermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(CommunityPermission::from_bits_retain(value as u32))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        if let Some(permission) = CommunityPermission::from_bits(value as u32) {
            Ok(permission)
        } else {
            Ok(CommunityPermission::from_bits_retain(value as u32))
        }
    }
}

impl<'de> Deserialize<'de> for CommunityPermission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JournalPermissionVisitor)
    }
}

impl CommunityPermission {
    /// Join two [`JournalPermission`]s into a single `u32`.
    pub fn join(lhs: CommunityPermission, rhs: CommunityPermission) -> CommunityPermission {
        lhs | rhs
    }

    /// Check if the given `input` contains the given [`JournalPermission`].
    pub fn check(self, permission: CommunityPermission) -> bool {
        if (self & CommunityPermission::ADMINISTRATOR) == CommunityPermission::ADMINISTRATOR {
            // has administrator permission, meaning everything else is automatically true
            return true;
        } else if (self & CommunityPermission::BANNED) == CommunityPermission::BANNED {
            // has banned permission, meaning everything else is automatically false
            return false;
        }

        (self & permission) == permission
    }

    /// Check if the given [`JournalPermission`] qualifies as "Member" status.
    pub fn check_member(self) -> bool {
        self.check(CommunityPermission::MEMBER)
    }

    /// Check if the given [`JournalPermission`] qualifies as "Moderator" status.
    pub fn check_moderator(self) -> bool {
        self.check(CommunityPermission::MANAGE_POSTS)
    }
}

impl Default for CommunityPermission {
    fn default() -> Self {
        Self::DEFAULT | Self::MEMBER
    }
}
