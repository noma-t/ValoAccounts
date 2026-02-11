use serde::Deserialize;
use std::collections::HashMap;
use wmi::{COMLibrary, Variant, WMIConnection};

#[derive(Deserialize, Debug)]
struct WmiProcessEvent {
    #[serde(rename = "__CLASS")]
    class: String,
}

fn check_process_running(wmi_con: &WMIConnection, process_name: &str) -> bool {
    let query = format!(
        "SELECT Name FROM Win32_Process WHERE Name = '{}'",
        process_name
    );
    match wmi_con.raw_query::<HashMap<String, Variant>>(&query) {
        Ok(results) => !results.is_empty(),
        Err(e) => {
            eprintln!("[ERROR] Query failed: {}", e);
            false
        }
    }
}

fn main() {
    println!("=== RiotClientServices.exe WMI Monitor ===");
    println!("Ctrl+C で終了\n");

    let com_lib = match COMLibrary::new() {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("[ERROR] COM 初期化失敗: {}", e);
            std::process::exit(1);
        }
    };

    let wmi_con = match WMIConnection::new(com_lib) {
        Ok(con) => con,
        Err(e) => {
            eprintln!("[ERROR] WMI 接続失敗: {}", e);
            std::process::exit(1);
        }
    };

    let running = check_process_running(&wmi_con, "RiotClientServices.exe");
    println!(
        "[現在の状態] RiotClientServices.exe: {}",
        if running { "起動中" } else { "停止中" }
    );
    println!("\nWMI イベント監視を開始...\n");

    let query = "SELECT * FROM __InstanceOperationEvent WITHIN 1 \
                 WHERE TargetInstance ISA 'Win32_Process' \
                 AND TargetInstance.Name = 'RiotClientServices.exe'";

    let iter = match wmi_con.raw_notification::<WmiProcessEvent>(query) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("[ERROR] WMI 通知の作成失敗: {}", e);
            std::process::exit(1);
        }
    };

    for event in iter {
        match event {
            Ok(data) => {
                let timestamp = chrono::Local::now().format("%H:%M:%S");
                match data.class.as_str() {
                    "__InstanceCreationEvent" => {
                        println!("[{}] 起動検知: RiotClientServices.exe が起動しました", timestamp);
                    }
                    "__InstanceDeletionEvent" => {
                        println!("[{}] 停止検知: RiotClientServices.exe が停止しました", timestamp);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("[ERROR] WMI イベントエラー: {}", e);
            }
        }
    }
}
