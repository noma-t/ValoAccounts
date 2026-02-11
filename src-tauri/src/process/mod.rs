use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use wmi::{COMLibrary, Variant, WMIConnection};

static RIOT_CLIENT_RUNNING: OnceLock<AtomicBool> = OnceLock::new();
static VALORANT_RUNNING: OnceLock<AtomicBool> = OnceLock::new();

fn query_process_running(wmi_con: &WMIConnection, process_name: &str) -> bool {
    let query = format!(
        "SELECT Name FROM Win32_Process WHERE Name = '{}'",
        process_name
    );
    match wmi_con.raw_query::<HashMap<String, Variant>>(&query) {
        Ok(results) => !results.is_empty(),
        Err(_) => false,
    }
}

fn check_process_running(process_name: &str) -> bool {
    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(_) => return false,
    };
    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(con) => con,
        Err(_) => return false,
    };
    query_process_running(&wmi_con, process_name)
}

pub fn check_riot_client_running() -> bool {
    RIOT_CLIENT_RUNNING
        .get()
        .map(|a| a.load(Ordering::Relaxed))
        .unwrap_or_else(|| check_process_running("RiotClientServices.exe"))
}

pub fn check_valorant_running() -> bool {
    VALORANT_RUNNING
        .get()
        .map(|a| a.load(Ordering::Relaxed))
        .unwrap_or_else(|| check_process_running("VALORANT-Win64-Shipping.exe"))
}

pub fn kill_riot_client() -> Result<(), String> {
    let output = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "RiotClientServices.exe"])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("taskkill failed: {}", stderr))
    }
}

pub fn launch_riot_client() -> Result<(), String> {
    use crate::db::get_settings;

    // Try to get path from settings first
    let mut candidates = Vec::new();

    if let Ok(settings) = get_settings() {
        if let Some(path) = settings.riot_client_service_path {
            if !path.is_empty() {
                candidates.push(path);
            }
        }
    }

    // Fallback to common paths
    candidates.extend([
        r"C:\Riot Games\Riot Client\RiotClientServices.exe".to_string(),
        r"C:\Program Files\Riot Games\Riot Client\RiotClientServices.exe".to_string(),
        r"C:\Program Files (x86)\Riot Games\Riot Client\RiotClientServices.exe".to_string(),
    ]);

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            std::process::Command::new(path)
                .spawn()
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    Err("Riot Client executable not found".to_string())
}

pub fn start_process_monitor(app_handle: AppHandle) {
    RIOT_CLIENT_RUNNING
        .get_or_init(|| AtomicBool::new(check_process_running("RiotClientServices.exe")));
    VALORANT_RUNNING
        .get_or_init(|| AtomicBool::new(check_process_running("VALORANT-Win64-Shipping.exe")));

    std::thread::spawn(move || {
        let com_lib = match COMLibrary::new() {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("Failed to initialize COM for process monitor: {}", e);
                return;
            }
        };
        let wmi_con = match WMIConnection::new(com_lib) {
            Ok(con) => con,
            Err(e) => {
                eprintln!("Failed to connect to WMI for process monitor: {}", e);
                return;
            }
        };

        loop {
            std::thread::sleep(Duration::from_secs(2));

            let riot_now = query_process_running(&wmi_con, "RiotClientServices.exe");
            let riot_prev = RIOT_CLIENT_RUNNING
                .get()
                .unwrap()
                .swap(riot_now, Ordering::Relaxed);
            if riot_now != riot_prev {
                if let Err(e) = app_handle.emit("riot-client-status", riot_now) {
                    eprintln!("Failed to emit riot-client-status: {}", e);
                }
            }

            let valo_now = query_process_running(&wmi_con, "VALORANT-Win64-Shipping.exe");
            let valo_prev = VALORANT_RUNNING
                .get()
                .unwrap()
                .swap(valo_now, Ordering::Relaxed);
            if valo_now != valo_prev {
                if let Err(e) = app_handle.emit("valorant-status", valo_now) {
                    eprintln!("Failed to emit valorant-status: {}", e);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wmi_connection() {
        let com_lib = COMLibrary::new();
        assert!(com_lib.is_ok(), "COM library should initialize successfully");

        let wmi_con = WMIConnection::new(com_lib.unwrap());
        assert!(wmi_con.is_ok(), "WMI connection should succeed");
    }

    #[test]
    fn test_check_known_process_running() {
        let com_lib = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_lib).unwrap();
        assert!(
            query_process_running(&wmi_con, "explorer.exe"),
            "explorer.exe should be detected as running"
        );
    }

    #[test]
    fn test_check_nonexistent_process() {
        let com_lib = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_lib).unwrap();
        assert!(
            !query_process_running(&wmi_con, "this_process_xyz_does_not_exist_123.exe"),
            "Non-existent process should return false"
        );
    }

    #[test]
    fn test_check_riot_client_running_does_not_panic() {
        let _ = check_riot_client_running();
    }
}
