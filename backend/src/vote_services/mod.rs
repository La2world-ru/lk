use serde::{Deserialize, Serialize};

pub mod mmotop;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct MmotopRecordId(pub u32);

impl Default for MmotopRecordId {
    fn default() -> Self {
        MmotopRecordId(251164611)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct VoteOptions {
    #[serde(rename = "_id")]
    pub id: u32,
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
