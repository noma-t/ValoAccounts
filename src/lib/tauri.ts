import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
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

export async function copyAccountPassword(accountId: number): Promise<void> {
  return invoke('copy_account_password', { accountId })
}

export interface RiotCookies {
  asid: string | null
  ccid: string | null
  clid: string | null
  sub: string | null
  csid: string | null
  ssid: string | null
  tdid: string | null
}

export async function getAccountCookies(accountId: number): Promise<RiotCookies | null> {
  return invoke('get_account_cookies', { accountId })
}

export interface DailyOffer {
  skin_uuid: string
  vp_cost: number
}

export interface NightMarketOffer {
  skin_uuid: string
  base_cost: number
  discount_cost: number
  discount_percent: number
}

export interface BundleItem {
  skin_uuid: string
  base_cost: number
  discounted_cost: number
  /** 0–100 */
  discount_percent: number
}

export interface Bundle {
  name: string
  total_base_cost: number
  total_discounted_cost: number
  /** 0–100 */
  total_discount_percent: number
  bundle_remaining_secs: number
  items: BundleItem[]
}

export interface Storefront {
  bundles?: Bundle[]
  daily_offers: DailyOffer[]
  daily_remaining_secs: number
  night_market: NightMarketOffer[] | null
  night_market_remaining_secs: number | null
}

export async function getShop(accountId: number, cookies: RiotCookies): Promise<Storefront> {
  return invoke('get_shop', { accountId, cookies })
}

export interface SkinWeapon {
  uuid: string
  display_name: string
  display_icon: string | null
  tier_uuid: string | null
  tier_color: string | null
  tier_rank: number | null
  tier_icon: string | null
}

export async function getSkinInfo(levelUuid: string): Promise<SkinWeapon | null> {
  return invoke('get_skin_info', { levelUuid })
}

export async function getSkinInfoBatch(levelUuids: string[]): Promise<(SkinWeapon | null)[]> {
  return invoke('get_skin_info_batch', { levelUuids })
}

export async function syncSkins(): Promise<boolean> {
  return invoke('sync_skins')
}

export async function openShopWindow(accountId: number, title: string): Promise<void> {
  return invoke('open_shop_window', { accountId, title })
}

export async function isDemoMode(): Promise<boolean> {
  return invoke('is_demo_mode')
}

export async function openTrackerProfile(riotId: string, tagline: string): Promise<void> {
  const url = `https://tracker.gg/valorant/profile/riot/${encodeURIComponent(`${riotId}#${tagline}`)}`
  return openUrl(url)
}
