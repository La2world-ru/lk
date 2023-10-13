use crate::vote_services::MmotopRecordId;
use crate::CONFIG;
use std::net::IpAddr;
use std::str::FromStr;

pub struct MmotopScrapper {
    pub last_id: MmotopRecordId,
}

impl MmotopScrapper {
    pub async fn scrap(&mut self) -> anyhow::Result<Vec<MmotopRecord>> {
        let body = reqwest::get(&CONFIG.mmotop_url).await?.text().await?;

        let mut max_id = self.last_id.0;

        let r = MmotopRecord::from_body(&body)
            .into_iter()
            .filter(|f| {
                max_id = max_id.max(f.record_id);

                f.record_id > self.last_id.0
            })
            .collect();

        self.last_id.0 = max_id;

        Ok(r)
    }
}

#[derive(Debug, Clone)]
pub struct MmotopRecord {
    pub record_id: u32,
    pub date: String,
    pub ip: IpAddr,
    pub name: String,
    pub count: u32,
}

impl MmotopRecord {
    fn from_body(body: &str) -> Vec<Self> {
        body.split('\n')
            .map(MmotopRecord::from_str)
            .filter_map(|f| f.ok())
            .collect()
    }
}

impl FromStr for MmotopRecord {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut c = s.split('\t');

        let Some(record_id) = c.next() else {
            return Err(());
        };
        let Ok(record_id) = u32::from_str(record_id) else {
            return Err(());
        };

        let Some(time) = c.next() else { return Err(()) };

        let Some(ip) = c.next() else { return Err(()) };
        let Ok(ip) = IpAddr::from_str(ip) else {
            return Err(());
        };

        let Some(name) = c.next() else { return Err(()) };

        let Some(count) = c.next() else {
            return Err(());
        };
        let Ok(count) = u32::from_str(count) else {
            return Err(());
        };

        Ok(Self {
            record_id,
            date: time.to_string(),
            ip,
            name: name.to_string(),
            count,
        })
    }
}
