use serde::{Deserialize, Serialize};

pub mod mmotop;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
pub struct MmotopRecordId(pub(crate) u32);

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct VoteOptions {
    #[serde(rename = "_id")]
    pub(crate) id: u32,
    #[serde(default)]
    pub last_mmotop_id: MmotopRecordId,
}

impl Default for VoteOptions {
    fn default() -> Self {
        Self {
            id: 1,
            last_mmotop_id: MmotopRecordId::default(),
        }
    }
}
