use anyhow::Result;
use serde::Deserialize;
use regex::Regex;

const DHL_JSON_REGEXP: &str = r#".*initialState: JSON\.parse\((.+")\).*"#;

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

    pub id: String,
    
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

/// Returns a DHLPackageStatus struct. You should check if it actually has any data with: `DHLPackageStatus.items[0].package_not_found.has_some()`  
/// `DHLPackageItem.package_not_found` will only be set if no package was found for that tracking code.
/// 
/// # Arguments
/// 
/// * `package_id` - Tracking code as `&str` of the parcel you wish to query, usually a number
/// 
/// # Examples
/// 
/// ``` ignore
/// use dhl_api::get_dhl_package_status;
/// let status = get_dhl_package_status("123456789").await?;
/// for item in status.items {
///     if item.package_not_found.is_some() {
///         // This item was not found
/// 
///         let why_not_found = item.package_not_found.unwrap();
///         // if why_not_found.no_data_available { ...
///         // if why_not_found.not_a_dhl_package { ...
/// 
///         continue;
///     }
/// 
///     let tracking_code = &item.id;
/// 
///     if item.has_complete_details {
///         let details = &item.item_details;
/// 
///         for event in &details.history.events.unwrap() {
///             // Do whatever you need 🦈
///         }
///     }
/// }
/// ```
pub async fn get_dhl_package_status(package_id: &str) -> Result<DHLPackageStatus, anyhow::Error> {
    let my_url = format!("https://www.dhl.de/int-verfolgen/?lang=en&domain=de&lang=en&domain=de&lang=en&domain=de&lang=en&domain=de&piececode={}", package_id);
    let body = reqwest::get(&my_url)
        .await?
        .text().await?;

    find_and_derez_json(&body)
}

pub fn get_dhl_package_status_from_str(body: &str) -> Result<DHLPackageStatus,anyhow::Error> {
    find_and_derez_json(&body)
}
