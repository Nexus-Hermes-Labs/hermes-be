use std::str::FromStr;

// ============================================
// DM PRIVACY
// ============================================

/// Who can send direct messages to the user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DmPrivacy {
    /// Anyone can send DMs
    Everyone,

    /// Only friends can send DMs
    #[default]
    Friends,

    /// Only server members can send DMs
    ServerMembers,

    /// No one can send DMs
    None,
}

impl DmPrivacy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Everyone => "everyone",
            Self::Friends => "friends",
            Self::ServerMembers => "server_members",
            Self::None => "none",
        }
    }
}

impl FromStr for DmPrivacy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "everyone" => Ok(Self::Everyone),
            "friends" => Ok(Self::Friends),
            "server_members" => Ok(Self::ServerMembers),
            "none" => Ok(Self::None),
            _ => Err(format!("Invalid DM privacy: {}", s)),
        }
    }
}

// ============================================
// FRIEND REQUEST PRIVACY
// ============================================

/// Who can send friend requests to the user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FriendRequestPrivacy {
    /// Anyone can send friend requests
    #[default]
    Everyone,

    /// Only friends of friends can send requests
    FriendsOfFriends,

    /// No one can send friend requests
    None,
}

impl FriendRequestPrivacy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Everyone => "everyone",
            Self::FriendsOfFriends => "friends_of_friends",
            Self::None => "none",
        }
    }
}

impl FromStr for FriendRequestPrivacy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "everyone" => Ok(Self::Everyone),
            "friends_of_friends" => Ok(Self::FriendsOfFriends),
            "none" => Ok(Self::None),
            _ => Err(format!("Invalid friend request privacy: {}", s)),
        }
    }
}

// ============================================
// CONTENT FILTER LEVEL
// ============================================

/// Content filtering strictness level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContentFilterLevel {
    /// No filtering
    Off = 0,

    /// Medium filtering (default)
    #[default]
    Medium = 1,

    /// Strict filtering
    Strict = 2,
}

impl ContentFilterLevel {
    pub fn as_i16(&self) -> i16 {
        *self as i16
    }

    pub fn from_i16(value: i16) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Off),
            1 => Ok(Self::Medium),
            2 => Ok(Self::Strict),
            _ => Err(format!("Invalid content filter level: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dm_privacy_parsing() {
        assert_eq!(
            "everyone".parse::<DmPrivacy>().unwrap(),
            DmPrivacy::Everyone
        );
        assert_eq!("friends".parse::<DmPrivacy>().unwrap(), DmPrivacy::Friends);
        assert_eq!(
            "server_members".parse::<DmPrivacy>().unwrap(),
            DmPrivacy::ServerMembers
        );
        assert_eq!("none".parse::<DmPrivacy>().unwrap(), DmPrivacy::None);
    }

    #[test]
    fn test_friend_request_privacy_parsing() {
        assert_eq!(
            "everyone".parse::<FriendRequestPrivacy>().unwrap(),
            FriendRequestPrivacy::Everyone
        );
        assert_eq!(
            "friends_of_friends"
                .parse::<FriendRequestPrivacy>()
                .unwrap(),
            FriendRequestPrivacy::FriendsOfFriends
        );
        assert_eq!(
            "none".parse::<FriendRequestPrivacy>().unwrap(),
            FriendRequestPrivacy::None
        );
    }

    #[test]
    fn test_content_filter_level() {
        assert_eq!(
            ContentFilterLevel::from_i16(0).unwrap(),
            ContentFilterLevel::Off
        );
        assert_eq!(
            ContentFilterLevel::from_i16(1).unwrap(),
            ContentFilterLevel::Medium
        );
        assert_eq!(
            ContentFilterLevel::from_i16(2).unwrap(),
            ContentFilterLevel::Strict
        );
        assert!(ContentFilterLevel::from_i16(3).is_err());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(DmPrivacy::default(), DmPrivacy::Friends);
        assert_eq!(
            FriendRequestPrivacy::default(),
            FriendRequestPrivacy::Everyone
        );
        assert_eq!(ContentFilterLevel::default(), ContentFilterLevel::Medium);
    }
}
