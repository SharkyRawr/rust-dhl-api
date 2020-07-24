use anyhow::Result;
use serde::Deserialize;
use regex::Regex;

const DHL_JSON_REGEXP: &str = r#".*initialState: JSON\.parse\((.+")\).*"#;

mod deserialize_as_thingie {
    use serde_json;
    use serde::de::{Deserialize, DeserializeOwned, Deserializer};
    
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let j = String::deserialize(deserializer)?;
        serde_json::from_str(&j).map_err(Error::custom)
    }
}

#[derive(Deserialize, Debug)]
pub struct DHLPackageItemHistoryEvent {
    #[serde(rename = "datum")]
    pub date: String,
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "ruecksendung")]
    pub return_shipment: bool,
    #[serde(rename = "ort")]
    pub location: Option<String>
}

#[derive(Deserialize, Debug)]
pub struct DHLPackageItemHistory {
    #[serde(rename = "events")]
    pub events: Option<Vec<DHLPackageItemHistoryEvent>>,
    #[serde(rename = "aktuellerStatus")]
    pub current_status: Option<String>,
    #[serde(rename = "fortschritt")]
    pub steps: u64,
}

#[derive(Deserialize, Debug)]
pub struct DHLPackageNotFoundInfo {
    #[serde(rename = "keineDatenVerfuegbar")]
    pub no_data_available: bool,
    #[serde(rename = "keineDhlPaketSendung")]
    pub not_a_dhl_package: bool
}

#[derive(Deserialize, Debug)]
pub struct DHLPackageItemDetails {
    #[serde(rename = "sendungsverlauf")]
    pub history: DHLPackageItemHistory,
    #[serde(rename = "zielland")]
    pub destination_country: Option<String>,
}
#[derive(Deserialize, Debug)]
pub struct DHLPackageItem {
    #[serde(with = "deserialize_as_thingie")]
    pub id: u64,
    
    #[serde(rename = "hasCompleteDetails")]
    pub has_complete_details: bool,

    #[serde(rename = "sendungsdetails")]
    pub item_details: DHLPackageItemDetails,

    #[serde(rename = "sendungNichtGefunden")]
    pub package_not_found: Option<DHLPackageNotFoundInfo>
}

#[derive(Deserialize, Debug)]
pub struct DHLPackageStatus {
    #[serde(rename = "sendungen")]
    pub items: Vec<DHLPackageItem>
}

pub fn find_and_derez_json(body: &str) -> Result<DHLPackageStatus> {
    let rex = Regex::new(DHL_JSON_REGEXP).unwrap();

    let caps = rex.captures(&body).unwrap();

    let json_escaped = &caps[1];
    let json = json_escaped
        .trim()
        [1..json_escaped.len()-1]
        .replace(r#"\""#, "\"");
    
    //println!("{}", json);

    let r: DHLPackageStatus = serde_json::from_str(&json).unwrap();
    Ok(r)
}

pub async fn get_dhl_package_status(package_id: u64) -> Result<DHLPackageStatus, anyhow::Error> {
    let my_url = format!("https://www.dhl.de/int-verfolgen/?lang=en&domain=de&lang=en&domain=de&lang=en&domain=de&lang=en&domain=de&piececode={}", package_id);
    let body = reqwest::get(&my_url)
        .await?
        .text().await?;

    find_and_derez_json(&body)
}

pub fn get_dhl_package_status_from_str(body: &str) -> Result<DHLPackageStatus,anyhow::Error> {
    find_and_derez_json(&body)
}
