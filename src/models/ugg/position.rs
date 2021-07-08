use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, strum::Display, strum::EnumString, Eq, PartialEq, Serialize, Deserialize,
)]
pub enum Position {
    Unknown = 0,
    Jungle = 1,
    Support = 2,
    Bottom = 3,
    Top = 4,
    Mid = 5,
}

impl From<isize> for Position {
    fn from(v: isize) -> Self {
        match v {
            x if x == Position::Jungle as isize => Position::Jungle,
            x if x == Position::Support as isize => Position::Support,
            x if x == Position::Bottom as isize => Position::Bottom,
            x if x == Position::Top as isize => Position::Top,
            x if x == Position::Mid as isize => Position::Mid,
            _ => Position::Unknown,
        }
    }
}
