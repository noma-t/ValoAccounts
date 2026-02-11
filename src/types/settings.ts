export interface Settings {
  id: number
  active_account_id: number | null
  riot_client_service_path: string | null
  riot_client_data_path: string | null
  account_data_path: string | null
  henrikdev_api_key: string | null
  launched: boolean
  created_at: string
  updated_at: string
}

export interface UpdateSettings {
  riot_client_service_path?: string | null
  riot_client_data_path?: string | null
  account_data_path?: string | null
  henrikdev_api_key?: string | null
}
