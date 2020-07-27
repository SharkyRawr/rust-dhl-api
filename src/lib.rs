use anyhow::Result;
use regex::Regex;
use serde::Deserialize;

const DHL_JSON_REGEXP: &str = r#".*initialState: JSON\.parse\((.+")\).*"#;

/// Parcel history events (where did you come from, where did you go?)
#[derive(Deserialize, Debug)]
pub struct DHLPackageItemHistoryEvent {
    /// Date
    #[serde(rename = "datum")]
    pub date: String,
    /// Status text
    #[serde(rename = "status")]
    pub status: String,
    /// If it was returned (?)
    #[serde(rename = "ruecksendung")]
    pub return_shipment: bool,
    /// Location where this happened
    #[serde(rename = "ort")]
    pub location: Option<String>,
}

/// Parcel history
#[derive(Deserialize, Debug)]
pub struct DHLPackageItemHistory {
    /// The parcel may or may not have made a few stops already, this Vec may contain them.
    #[serde(rename = "events")]
    pub events: Option<Vec<DHLPackageItemHistoryEvent>>,
    /// Current state of the parcel
    #[serde(rename = "aktuellerStatus")]
    pub current_status: Option<String>,
    /// Number of steps there are (or should be?) - might as well use `events.len()` 🤷‍♀️
    #[serde(rename = "fortschritt")]
    pub steps: u64,
}

/// Information why a parcel tracking code may not have been found.
/// This is only set when no tracking information was found (yet?).
#[derive(Deserialize, Debug)]
pub struct DHLPackageNotFoundInfo {
    /// If `true`, no data is available for this tracking code.
    #[serde(rename = "keineDatenVerfuegbar")]
    pub no_data_available: bool,
    /// If `true`, this is probably not a DHL tracking code.
    #[serde(rename = "keineDhlPaketSendung")]
    pub not_a_dhl_package: bool,
}

/// Parcel item details
#[derive(Deserialize, Debug)]
pub struct DHLPackageItemDetails {
    /// History of the parcel
    #[serde(rename = "sendungsverlauf")]
    pub history: DHLPackageItemHistory,
    /// Destination country of the parcel
    #[serde(rename = "zielland")]
    pub destination_country: Option<String>,
}

/// Parcel item elements
#[derive(Deserialize, Debug)]
pub struct DHLPackageItem {
    /// Tracking code, just for keeping track.
    pub id: String,

    /// (unsure what this means, if you know let me know ❤️)
    #[serde(rename = "hasCompleteDetails")]
    pub has_complete_details: bool,

    /// Details for this parcel
    #[serde(rename = "sendungsdetails")]
    pub item_details: DHLPackageItemDetails,

    /// If no parcel was found, this variable is **not `None`** and may contains reasons as to why.
    /// Otherwise it should be `None`.
    #[serde(rename = "sendungNichtGefunden")]
    pub package_not_found: Option<DHLPackageNotFoundInfo>,
}

/// Root element, contains a vector with parcel items
#[derive(Deserialize, Debug)]
pub struct DHLPackageStatus {
    /// The items returned for the query.
    #[serde(rename = "sendungen")]
    pub items: Vec<DHLPackageItem>,
}

fn find_and_derez_json(body: &str) -> Result<DHLPackageStatus, anyhow::Error> {
    let rex = Regex::new(DHL_JSON_REGEXP).unwrap();

    let caps = rex.captures(&body).unwrap();

    let json_escaped = &caps[1];
    let json = json_escaped.trim()[1..json_escaped.len() - 1].replace(r#"\""#, "\"");

    //println!("{}", json);

    let r: DHLPackageStatus = serde_json::from_str(&json)?;
    Ok(r)
}

/// Returns a DHLPackageStatus struct. You should check if it actually has any data with: `DHLPackageStatus.items[0].package_not_found.has_some()`  
/// `DHLPackageItem.package_not_found` will only be set if no package was found for that tracking code.
///
/// # Arguments
///
/// * `package_id` - Tracking code as `&str` of the parcel you wish to query, usually a number but sometimes contains letters.
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
    let body = reqwest::get(&my_url).await?.text().await?;

    find_and_derez_json(&body)
}

/// Try to parse a HTML body and look for the shipping information, usually you need not to call this
/// unless you wish to use your own HTTP client or backend URL or whatever other wizardry you're into.
/// 
/// # Arguments
/// 
/// `htmlbody` - HTML code to scan, check out the sourcecode to find the regexp and what it expects.
pub fn get_dhl_package_from_html(htmlbody: &str) -> Result<DHLPackageStatus, anyhow::Error> {
    find_and_derez_json(&htmlbody)
}
