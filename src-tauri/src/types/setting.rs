use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSetting {
    pub search: SearchSetting,
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchSetting {
    pub fuzziness: SearchFuzziness,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, specta::Type, Default)]
pub enum SearchFuzziness {
    #[default]
    Exact,
    Fuzzy,
    Fuzziest,
}

impl SearchFuzziness {
    pub fn levenshtein_distance(&self) -> u8 {
        match self {
            SearchFuzziness::Exact => 0,
            SearchFuzziness::Fuzzy => 1,
            SearchFuzziness::Fuzziest => 2,
        }
    }
}
