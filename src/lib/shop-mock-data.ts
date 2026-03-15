import type { Storefront, SkinWeapon } from './tauri'
import { ITEM_TYPE_SKIN, ITEM_TYPE_SPRAY, ITEM_TYPE_BUDDY, ITEM_TYPE_PLAYERCARD, ITEM_TYPE_TITLE } from './tauri'
import type { ItemInfo } from './shop-utils'

export const MOCK_SKIN_MAP: Record<string, SkinWeapon> = {
  // Bundle: Spectrum (5 items)
  'mock-sp-1': { uuid: 'mock-sp-1', display_name: 'Spectrum Phantom', display_icon: null, tier_color: '0096FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-sp-2': { uuid: 'mock-sp-2', display_name: 'Spectrum Vandal', display_icon: null, tier_color: '0096FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-sp-3': { uuid: 'mock-sp-3', display_name: 'Spectrum Operator', display_icon: null, tier_color: '0096FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-sp-4': { uuid: 'mock-sp-4', display_name: 'Spectrum Sheriff', display_icon: null, tier_color: '0096FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-sp-5': { uuid: 'mock-sp-5', display_name: 'Spectrum Knife', display_icon: null, tier_color: '0096FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  // Bundle: Ruination (3 items)
  'mock-ru-1': { uuid: 'mock-ru-1', display_name: 'Ruination Phantom', display_icon: null, tier_color: '9147FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-ru-2': { uuid: 'mock-ru-2', display_name: 'Ruination Vandal', display_icon: null, tier_color: '9147FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-ru-3': { uuid: 'mock-ru-3', display_name: 'Ruination Knife', display_icon: null, tier_color: '9147FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  // Daily (4 items)
  'mock-a': { uuid: 'mock-a', display_name: 'DEMO Phantom', display_icon: null, tier_color: 'FF4655', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-b': { uuid: 'mock-b', display_name: 'DEMO Vandal', display_icon: null, tier_color: '009BDE', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-c': { uuid: 'mock-c', display_name: 'DEMO Operator', display_icon: null, tier_color: 'F5A623', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-d': { uuid: 'mock-d', display_name: 'DEMO Knife', display_icon: null, tier_color: 'BD3944', tier_uuid: null, tier_rank: null, tier_icon: null },
  // Nightmarket (6 items)
  'mock-nm-1': { uuid: 'mock-nm-1', display_name: 'Prime Phantom', display_icon: null, tier_color: 'F0C75E', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-nm-2': { uuid: 'mock-nm-2', display_name: 'Ion Vandal', display_icon: null, tier_color: '5CFFCB', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-nm-3': { uuid: 'mock-nm-3', display_name: 'Elderflame Operator', display_icon: null, tier_color: 'FF6B35', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-nm-4': { uuid: 'mock-nm-4', display_name: 'Glitchpop Frenzy', display_icon: null, tier_color: 'FF00FF', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-nm-5': { uuid: 'mock-nm-5', display_name: 'Reaver Sheriff', display_icon: null, tier_color: 'E74C3C', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-nm-6': { uuid: 'mock-nm-6', display_name: 'Origin Guardian', display_icon: null, tier_color: '7B68EE', tier_uuid: null, tier_rank: null, tier_icon: null },
}

export const MOCK_ITEM_MAP: Record<string, ItemInfo | null> = {
  'mock-acc-spray-1': { kind: 'spray', data: { uuid: 'mock-acc-spray-1', display_name: 'DEMO Spray 1', display_icon: null, full_transparent_icon: null, animation_gif: null, asset_path: null, level_uuid: 'mock-acc-spray-1', spray_level: null } },
  'mock-acc-spray-2': { kind: 'spray', data: { uuid: 'mock-acc-spray-2', display_name: 'DEMO Spray 2', display_icon: null, full_transparent_icon: null, animation_gif: null, asset_path: null, level_uuid: 'mock-acc-spray-2', spray_level: null } },
  'mock-acc-buddy-1': { kind: 'buddy', data: { uuid: 'mock-acc-buddy-1', display_name: 'DEMO Buddy 1', display_icon: null, asset_path: null, level_uuid: 'mock-acc-buddy-1', charm_level: null } },
  'mock-acc-buddy-2': { kind: 'buddy', data: { uuid: 'mock-acc-buddy-2', display_name: 'DEMO Buddy 2', display_icon: null, asset_path: null, level_uuid: 'mock-acc-buddy-2', charm_level: null } },
  'mock-acc-card-1': { kind: 'playercard', data: { uuid: 'mock-acc-card-1', display_name: 'DEMO Player Card', display_icon: null, small_art: null, wide_art: null, large_art: null, asset_path: null } },
  'mock-acc-title-1': { kind: 'title', data: { uuid: 'mock-acc-title-1', display_name: 'DEMO Title', title_text: 'The Demo', asset_path: null } },
}

export const MOCK_STOREFRONT: Storefront = {
  bundles: [
    {
      name: 'Spectrum',
      total_base_cost: 14875,
      total_discounted_cost: 8825,
      total_discount_percent: 40.7,
      bundle_remaining_secs: 3600 * 72,
      items: [
        { item_uuid: 'mock-sp-1', item_type_id: ITEM_TYPE_SKIN, base_cost: 2175, discounted_cost: 1262, discount_percent: 42 },
        { item_uuid: 'mock-sp-2', item_type_id: ITEM_TYPE_SKIN, base_cost: 2175, discounted_cost: 1262, discount_percent: 42 },
        { item_uuid: 'mock-sp-3', item_type_id: ITEM_TYPE_SKIN, base_cost: 2175, discounted_cost: 1262, discount_percent: 42 },
        { item_uuid: 'mock-sp-4', item_type_id: ITEM_TYPE_SKIN, base_cost: 2175, discounted_cost: 1262, discount_percent: 42 },
        { item_uuid: 'mock-sp-5', item_type_id: ITEM_TYPE_SKIN, base_cost: 4350, discounted_cost: 2523, discount_percent: 42 },
        { item_uuid: 'mock-sp-spray', item_type_id: ITEM_TYPE_SPRAY, base_cost: 325, discounted_cost: 228, discount_percent: 30 },
        { item_uuid: 'mock-sp-buddy', item_type_id: ITEM_TYPE_BUDDY, base_cost: 475, discounted_cost: 333, discount_percent: 30 },
        { item_uuid: 'mock-sp-card', item_type_id: ITEM_TYPE_PLAYERCARD, base_cost: 375, discounted_cost: 263, discount_percent: 30 },
      ],
    },
    {
      name: 'Ruination',
      total_base_cost: 7100,
      total_discounted_cost: 4970,
      total_discount_percent: 30.0,
      bundle_remaining_secs: 3600 * 48,
      items: [
        { item_uuid: 'mock-ru-1', item_type_id: ITEM_TYPE_SKIN, base_cost: 1775, discounted_cost: 1243, discount_percent: 30 },
        { item_uuid: 'mock-ru-2', item_type_id: ITEM_TYPE_SKIN, base_cost: 1775, discounted_cost: 1243, discount_percent: 30 },
        { item_uuid: 'mock-ru-3', item_type_id: ITEM_TYPE_SKIN, base_cost: 3550, discounted_cost: 2485, discount_percent: 30 },
      ],
    },
  ],
  daily_offers: [
    { skin_uuid: 'mock-a', vp_cost: 1775 },
    { skin_uuid: 'mock-b', vp_cost: 2175 },
    { skin_uuid: 'mock-c', vp_cost: 3550 },
    { skin_uuid: 'mock-d', vp_cost: 1275 },
  ],
  daily_remaining_secs: 3600 * 8,
  accessories: [
    { item_uuid: 'mock-acc-spray-1', item_type_id: ITEM_TYPE_SPRAY, kc_cost: 375 },
    { item_uuid: 'mock-acc-spray-2', item_type_id: ITEM_TYPE_SPRAY, kc_cost: 375 },
    { item_uuid: 'mock-acc-buddy-1', item_type_id: ITEM_TYPE_BUDDY, kc_cost: 400 },
    { item_uuid: 'mock-acc-buddy-2', item_type_id: ITEM_TYPE_BUDDY, kc_cost: 400 },
    { item_uuid: 'mock-acc-card-1', item_type_id: ITEM_TYPE_PLAYERCARD, kc_cost: 500 },
    { item_uuid: 'mock-acc-title-1', item_type_id: ITEM_TYPE_TITLE, kc_cost: 500 },
  ],
  accessories_remaining_secs: 3600 * 24 * 3,
  night_market: [
    { skin_uuid: 'mock-nm-1', base_cost: 2175, discount_cost: 870, discount_percent: 60 },
    { skin_uuid: 'mock-nm-2', base_cost: 2175, discount_cost: 1305, discount_percent: 40 },
    { skin_uuid: 'mock-nm-3', base_cost: 2675, discount_cost: 1337, discount_percent: 50 },
    { skin_uuid: 'mock-nm-4', base_cost: 2175, discount_cost: 1740, discount_percent: 20 },
    { skin_uuid: 'mock-nm-5', base_cost: 1775, discount_cost: 533, discount_percent: 70 },
    { skin_uuid: 'mock-nm-6', base_cost: 1775, discount_cost: 1243, discount_percent: 30 },
  ],
  night_market_remaining_secs: 3600 * 24 * 5,
}
