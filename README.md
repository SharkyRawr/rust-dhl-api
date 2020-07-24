# Rust DHL API

**Rust DHL API** provides a way to query the public www.DHL.de package tracking website for information on parcels *from rust*. Yay!

This library might stop working at any time since it relies on HTTP requests and regexp. Only publicly available information can be queried.

## Example

```rust
use dhl_api::get_dhl_package_status;

let status = get_dhl_package_status("123456789").await?;
for item in status.items {
    if item.package_not_found.is_some() {
        // This item was not found

        let why_not_found = item.package_not_found.unwrap();
        // if why_not_found.no_data_available { ...
        // if why_not_found.not_a_dhl_package { ...

        continue;
    }

    let tracking_code = &item.id;

    if item.has_complete_details {
        let details = &item.item_details;

        for event in &details.history.events.unwrap() {
            // Do whatever you need 🦈
        }
    }
}
```
