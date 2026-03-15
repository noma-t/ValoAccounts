#[tauri::command]
pub fn get_skin_info(level_uuid: String) -> Result<Option<crate::skins::SkinWeapon>, String> {
    crate::skins::get_skin_by_level_uuid(&level_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_skin_info_batch(
    level_uuids: Vec<String>,
) -> Result<Vec<Option<crate::skins::SkinWeapon>>, String> {
    crate::skins::get_skins_by_level_uuids(&level_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_buddy_info(level_uuid: String) -> Result<Option<crate::skins::BuddyItem>, String> {
    crate::skins::get_buddy_by_level_uuid(&level_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_buddy_info_batch(
    level_uuids: Vec<String>,
) -> Result<Vec<Option<crate::skins::BuddyItem>>, String> {
    crate::skins::get_buddies_by_level_uuids(&level_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_flex_info(uuid: String) -> Result<Option<crate::skins::FlexItem>, String> {
    crate::skins::get_flex_by_uuid(&uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_flex_info_batch(uuids: Vec<String>) -> Result<Vec<Option<crate::skins::FlexItem>>, String> {
    crate::skins::get_flex_by_uuids(&uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_title_info_batch(
    uuids: Vec<String>,
) -> Result<Vec<Option<crate::skins::TitleItem>>, String> {
    crate::skins::get_titles_by_uuids(&uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playercard_info(uuid: String) -> Result<Option<crate::skins::PlayercardItem>, String> {
    crate::skins::get_playercard_by_uuid(&uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_playercard_info_batch(
    uuids: Vec<String>,
) -> Result<Vec<Option<crate::skins::PlayercardItem>>, String> {
    crate::skins::get_playercards_by_uuids(&uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_spray_info(level_uuid: String) -> Result<Option<crate::skins::SprayItem>, String> {
    crate::skins::get_spray_by_level_uuid(&level_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_spray_info_batch(
    level_uuids: Vec<String>,
) -> Result<Vec<Option<crate::skins::SprayItem>>, String> {
    crate::skins::get_sprays_by_level_uuids(&level_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_skins() -> Result<bool, String> {
    crate::skins::sync_skins_database()
        .await
        .map_err(|e| e.to_string())
}
