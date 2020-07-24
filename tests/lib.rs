#[cfg(test)]
mod tests {
    #[test]
    fn test_derez() {

        const EXAMPLE_BODY: &str = r#"
        <script>
  
    window.__INITIAL_APP_STATE__ = {
      initialState: JSON.parse("{\"sendungen\":[{\"id\":\"523361125086\",\"hasCompleteDetails\":true,\"sendungsinfo\":{\"gesuchteSendungsnummer\":\"523361125086\"},\"sendungsdetails\":{\"sendungsnummern\":{\"sendungsnummer\":\"523361125086\"},\"eigenschaften\":{},\"sendungsverlauf\":{\"datumAktuellerStatus\":\"2020-07-24T02:19:00+02:00\",\"aktuellerStatus\":\"Shipment sent directly from parcel center to business customer\",\"kurzStatus\":\"The shipment has been sent directly from the parcel center to the business customer.\",\"fortschritt\":5,\"farbe\":0,\"iconId\":\"5\",\"events\":[{\"datum\":\"2020-07-22T16:39:00+02:00\",\"status\":\"The shipment has been taken from the PACKSTATION for onward transportation\",\"ruecksendung\":false},{\"datum\":\"2020-07-23T15:39:00+02:00\",\"ort\":\"Hannover\",\"status\":\"The shipment has been processed in the parcel center\",\"ruecksendung\":false},{\"datum\":\"2020-07-23T15:40:00+02:00\",\"status\":\"Shipment sent directly from parcel center to business customer\",\"ruecksendung\":false},{\"datum\":\"2020-07-24T02:18:00+02:00\",\"ort\":\"Neumark\",\"status\":\"The shipment has been processed in the parcel center\",\"ruecksendung\":false},{\"datum\":\"2020-07-24T02:19:00+02:00\",\"status\":\"Shipment sent directly from parcel center to business customer\",\"ruecksendung\":false}]},\"services\":{\"statusbenachrichtigung\":{\"aktuellerStatus\":true,\"geplanteZustellung\":false,\"erfolgteZustellung\":false}},\"zustellung\":{\"showAbholcode\":false,\"abholcodeAvailable\":false,\"zugestelltAnPackstation\":false,\"benachrichtigtInFiliale\":false},\"zielland\":\"Germany\",\"istZugestellt\":true,\"ruecksendung\":false,\"retoure\":false,\"warenpost\":false,\"expressSendung\":false},\"versandDatumBenoetigt\":false}]}"),
      config: {"verfolgenDataPath":"/int-verfolgen/data","verfolgenContextPath":"/int-verfolgen","shipperIconPath":"/int-verfolgen/shippericons","assetPath":"/int-verfolgen/static/v900/","verfolgenVersion":"v900","currentDomain":"de","currentLanguage":"en","verfolgenCsrfToken":"56f42818-45bb-4275-adcc-4fa2be3fa41e","verfolgenBundleHash":"c6a995e6e77dd9599daa2b190b11e2c41ad2c847d17129301f456cd15cf72abb","verfolgenI18NHash":"5a9eeb116d2d22eb8683adf1d0b23d9ffd88a3f760e430074811fb02c739e5cf","portal":true,"zeroPercentPackageNotificationActive":true,"frontendDebug":false,"initialWG":0,"liveTrackingRefreshTime":1,"shipperIconClickedRefreshTime":10,"detailsViewNavigationRefreshTime":60,"renderMode":"WIDGET","showBanner":false,"packstationApiKey":"a0d5b9049ba8918871e6e20bd5c49974","searchLimit":10}
    };
    window.nol = {};
    window.nol.redirectRules = [{check: function (data) {             if (data.parameter && data.parameter.piececode) {               var shipmentId = data.parameter.piececode;               return Boolean(shipmentId.match(/(LX|LY|RX|RS)[0-9]{9}DE/gi));             }             return false;},destination: function (data) {return 'https://www.packet.deutschepost.com/web/portal-europe/packet_traceit?barcode=' +  data.parameter.piececode}}];
  
</script>
"#;

        let res = dhl_api::find_and_derez_json(&EXAMPLE_BODY).unwrap();
        let item = res.items.first().unwrap();
        println!("{:?}", item);

        assert_eq!(item.id, "523361125086");
        assert_eq!(item.has_complete_details, true);
        
        let item_details = &item.item_details;
        assert_eq!(item_details.destination_country.as_ref().unwrap(), "Germany");
        let history = &item_details.history;
        assert_eq!(history.steps, 5);
        let events = &history.events.as_ref().unwrap();
        assert_ne!(events.len(), 0);
        assert_eq!(events.first().unwrap().return_shipment, false);
    }

    #[tokio::test]
    async fn test_fetch() {
        let res = dhl_api::get_dhl_package_status("523361125086").await.unwrap();
        let item = res.items.first().unwrap();
        println!("{:?}", item);

        assert_eq!(item.id, "523361125086");
        assert_eq!(item.has_complete_details, true);

        let package_found = item.package_not_found.as_ref();
        if package_found.is_some() {
            // Skip the rest of the test because there is no data available for this package id
            if package_found.unwrap().no_data_available || package_found.unwrap().not_a_dhl_package {
                return;
            }
        }
        
        let item_details = &item.item_details;
        assert_eq!(item_details.destination_country.as_ref().unwrap(), "Germany");
        let history = &item_details.history;
        assert_eq!(history.steps, 5);
        let events = &history.events.as_ref().unwrap();
        assert_ne!(events.len(), 0);
        assert_eq!(events.first().unwrap().return_shipment, false);
    }
}
