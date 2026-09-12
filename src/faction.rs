use std::fmt::Display;

use colorize::AnsiColor;
use serde::{Deserialize, Serialize};

use Faction::Unaligned;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    TradeFederation,
    Blob,
    StarEmpire,
    MachineCult,
    Unaligned,
}

impl Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:?}", self);

        write!(
            f,
            "{}",
            match self {
                Faction::TradeFederation => s.blue(),
                Faction::Blob => s.green(),
                Faction::StarEmpire => s.yellow(),
                Faction::MachineCult => s.red(),
                Unaligned => s,
            }
        )
    }
}
