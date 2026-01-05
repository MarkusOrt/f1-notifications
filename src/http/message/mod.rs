#![allow(unused)]


#[allow(non_snake_case)]
pub mod Flags {
    pub const CROSSPOSTED: u32 = 1 << 0;
    pub const IS_CROSSPOST: u32 = 1 << 1;
    pub const SUPPRESS_EMBEDS: u32 = 1 << 2;
    pub const SOURCE_MESSAGE_DELETED: u32 = 1 << 3;
    pub const URGENT: u32 = 1 << 4;
    pub const HAS_THREAD: u32 = 1 << 5;
    pub const EPHEMERAL: u32 = 1 << 6;
    pub const LOADING: u32 = 1 << 7;
    pub const FAILED_TO_MENTION_SOME_ROLES_IN_THREAD: u32 = 1 << 8;
    pub const SUPPRESS_NOTIFICATIONS: u32 = 1 << 12;
    pub const IS_VOICE_MESSAGE: u32 = 1 << 13;
    pub const HAS_SNAPSHOT: u32 = 1 << 14;
    pub const IS_COMPONENTS_V2: u32 = 1 << 15;
}
