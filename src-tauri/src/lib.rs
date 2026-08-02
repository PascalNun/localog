#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppIdentity {
    name: &'static str,
    version: &'static str,
    local_only: bool,
}

#[tauri::command]
fn app_identity() -> AppIdentity {
    AppIdentity {
        name: "LocaLog",
        version: env!("CARGO_PKG_VERSION"),
        local_only: true,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_identity])
        .run(tauri::generate_context!())
        .expect("failed to run LocaLog");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_keeps_the_local_first_contract_visible() {
        let identity = app_identity();
        assert_eq!(identity.name, "LocaLog");
        assert!(identity.local_only);
    }
}
