import { invoke } from '@tauri-apps/api/core'
import type { Settings, UpdateSettings } from '../types/settings'
import type { Account, CreateAccount, UpdateAccount } from '../types/account'

export async function getAppDir(): Promise<string> {
  return invoke('get_app_dir')
}

export async function getDefaultRiotClientServicePath(): Promise<string> {
  return invoke('get_default_riot_client_service_path')
}

export async function getDefaultRiotClientDataPath(): Promise<string> {
  return invoke('get_default_riot_client_data_path')
}

export async function getSettings(): Promise<Settings> {
  return invoke('get_app_settings')
}

export async function updateSettings(settings: UpdateSettings): Promise<Settings> {
  return invoke('update_app_settings', { settings })
}

export async function addAccount(account: CreateAccount): Promise<Account> {
  return invoke('add_account', { account })
}

export async function listAccounts(): Promise<Account[]> {
  return invoke('list_accounts')
}

export async function updateAccount(account: UpdateAccount): Promise<Account> {
  return invoke('edit_account', { account })
}

export async function checkCurrentDataAvailable(): Promise<boolean> {
  return invoke('check_current_data_available')
}

export async function markLaunched(): Promise<void> {
  return invoke('mark_launched')
}

export async function getRiotClientStatus(): Promise<boolean> {
  return invoke('get_riot_client_status')
}

export async function getValorantStatus(): Promise<boolean> {
  return invoke('get_valorant_status')
}

export async function killRiotClient(): Promise<void> {
  return invoke('kill_riot_client')
}

export async function launchRiotClient(): Promise<void> {
  return invoke('launch_riot_client')
}

export async function switchAccount(accountId: number | null): Promise<void> {
  return invoke('switch_account', { accountId })
}
