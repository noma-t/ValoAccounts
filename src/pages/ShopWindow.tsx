import { useState, useEffect, useRef } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  getAccountCookies,
  getShop,
  getSkinInfoBatch,
  getBuddyInfoBatch,
  getPlayercardInfoBatch,
  getSprayInfoBatch,
  getFlexInfoBatch,
  isDemoMode,
  ITEM_TYPE_SKIN,
  ITEM_TYPE_BUDDY,
  ITEM_TYPE_PLAYERCARD,
  ITEM_TYPE_SPRAY,
} from '../lib/tauri'
import type {
  Storefront,
  SkinWeapon,
  BuddyItem,
  PlayercardItem,
  SprayItem,
  FlexItem,
  DailyOffer,
  NightMarketOffer,
  Bundle,
} from '../lib/tauri'
import '../App.css'

function formatCountdown(totalSecs: number): string {
  if (totalSecs <= 0) return '00:00:00'
  const d = Math.floor(totalSecs / 86400)
  const h = Math.floor((totalSecs % 86400) / 3600)
  const m = Math.floor((totalSecs % 3600) / 60)
  const s = totalSecs % 60
  const hms = `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return d > 0 ? `${d}d ${hms}` : hms
}

function useCountdown(initialSecs: number | null): number | null {
  const [remaining, setRemaining] = useState(initialSecs)
  const startRef = useRef<number | null>(null)
  const initialRef = useRef(initialSecs)

  useEffect(() => {
    initialRef.current = initialSecs
    if (initialSecs === null || initialSecs <= 0) {
      setRemaining(initialSecs)
      startRef.current = null
      return
    }

    startRef.current = Date.now()
    setRemaining(initialSecs)

    const id = setInterval(() => {
      const elapsed = Math.floor((Date.now() - (startRef.current ?? Date.now())) / 1000)
      const next = (initialRef.current ?? 0) - elapsed
      setRemaining(next > 0 ? next : 0)
    }, 1000)

    return () => clearInterval(id)
  }, [initialSecs])

  return remaining
}

function tierHex(tierColor: string | null): string | null {
  if (!tierColor) return null
  const hex = tierColor.slice(0, 6)
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) return null
  return hex
}

function cardGradient(hex: string | null): React.CSSProperties {
  if (!hex) {
    return { background: 'linear-gradient(to bottom, #404040 0%, #1a1a1a 100%)' }
  }
  return {
    background: `linear-gradient(to bottom, #${hex}40 0%, #1a1a1a 70%)`,
    borderBottom: `2px solid #${hex}90`,
  }
}


function VpIcon() {
  return <img src="/valo-icon.svg" alt="" width={12} height={12} className="opacity-70 block shrink-0" />
}

// Union type for any bundle item info resolved from the DB
type ItemInfo =
  | { kind: 'skin'; data: SkinWeapon }
  | { kind: 'buddy'; data: BuddyItem }
  | { kind: 'playercard'; data: PlayercardItem }
  | { kind: 'spray'; data: SprayItem }
  | { kind: 'flex'; data: FlexItem }

function skinImageUrl(skin: SkinWeapon | null, levelUuid: string): string {
  if (skin?.display_icon) return skin.display_icon
  return `https://media.valorant-api.com/weaponskinlevels/${levelUuid}/displayicon.png`
}

function formatVp(vp: number): string {
  return vp.toLocaleString()
}

// --- Mock data ---

const MOCK_SKIN_MAP: Record<string, SkinWeapon> = {
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

const MOCK_STOREFRONT: Storefront = {
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

// --- Components ---

interface SectionHeaderProps {
  label: string
  countdown?: number | null
}

function SectionHeader({ label, countdown }: SectionHeaderProps) {
  return (
    <div className="flex items-center gap-3 mb-3">
      <span className="text-xs font-bold uppercase tracking-widest text-neutral-300 shrink-0">
        {label}
      </span>
      {countdown != null && countdown >= 0 && (
        <span className="text-xs tabular-nums text-neutral-500 shrink-0">
          {formatCountdown(countdown)}
        </span>
      )}
      <div className="flex-1 h-px bg-neutral-700/60" />
    </div>
  )
}

interface BundleGroupProps {
  bundle: Bundle
  itemMap: Record<string, ItemInfo | null>
}

function BundleGroup({ bundle, itemMap }: BundleGroupProps) {
  const remaining = useCountdown(bundle.bundle_remaining_secs)

  // Classify by resolved itemMap kind rather than type_id, so items with unexpected
  // type IDs that resolve to flex are still shown as large cards.
  const largeItems = bundle.items.filter(
    (i) => i.item_type_id === ITEM_TYPE_SKIN || itemMap[i.item_uuid]?.kind === 'flex'
  )
  const bonusItems = bundle.items.filter(
    (i) => i.item_type_id !== ITEM_TYPE_SKIN && itemMap[i.item_uuid]?.kind !== 'flex'
  )

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-3">
        <span className="text-sm font-semibold text-white shrink-0">{bundle.name}</span>
        <div className="flex items-center gap-0.5 text-xs text-white/40">
          <VpIcon />
          <span className="line-through tabular-nums">{formatVp(bundle.total_base_cost)}</span>
        </div>
        <span className="text-xs font-semibold text-green-400">
          -{Math.round(bundle.total_discount_percent)}%
        </span>
        <div className="flex items-center gap-0.5 text-sm font-semibold text-white">
          <VpIcon />
          <span className="tabular-nums">{formatVp(bundle.total_discounted_cost)}</span>
        </div>
        {remaining !== null && (
          <span className="text-xs tabular-nums text-neutral-500">
            {formatCountdown(remaining)}
          </span>
        )}
      </div>
      <div className="flex gap-4 overflow-x-auto pb-1 shop-scrollbar min-h-[155px]">
        {largeItems.map((item) => {
          const info = itemMap[item.item_uuid] ?? null
          const skin = info?.kind === 'skin' ? info.data : null
          const flex = info?.kind === 'flex' ? info.data : null
          const hex = tierHex(skin?.tier_color ?? null)
          return (
            <div key={item.item_uuid} className="w-[276px] shrink-0">
              <SkinCard
                skin={skin}
                offer={{ skin_uuid: item.item_uuid, vp_cost: item.discounted_cost }}
                hex={hex}
                strikePrice={item.base_cost}
                discountPercent={Math.round(item.discount_percent)}
                fallbackName={flex?.display_name}
                fallbackIcon={flex?.display_icon}
              />
            </div>
          )
        })}
        {bonusItems.map((item) => (
          <BonusItemCard
            key={item.item_uuid}
            item={item}
            info={itemMap[item.item_uuid] ?? null}
          />
        ))}
      </div>
    </div>
  )
}

interface BonusItemCardProps {
  item: Bundle['items'][number]
  info: ItemInfo | null
}

function BonusItemCard({ item, info }: BonusItemCardProps) {
  let icon: string | null = null
  let name: string | null = null
  let label: string | null = null

  if (info?.kind === 'buddy') {
    icon = info.data.display_icon
    name = info.data.display_name
    label = 'Buddy'
  } else if (info?.kind === 'spray') {
    icon = info.data.full_transparent_icon ?? info.data.display_icon
    name = info.data.display_name
    label = 'Spray'
  } else if (info?.kind === 'playercard') {
    icon = info.data.display_icon
    name = info.data.display_name
    label = 'Card'
  } else if (info?.kind === 'flex') {
    icon = info.data.display_icon
    name = info.data.display_name
    label = 'Title'
  }

  return (
    <div className="w-[120px] shrink-0 rounded overflow-hidden bg-neutral-800/60 flex flex-col">
      <div className="flex-1 flex items-center justify-center p-3 min-h-0">
        {icon ? (
          <img
            src={icon}
            alt={name ?? ''}
            className="w-full h-full object-contain"
            loading="lazy"
            onError={(e) => { e.currentTarget.style.display = 'none' }}
          />
        ) : (
          <div className="w-full h-full rounded bg-neutral-700/50" />
        )}
      </div>
      <div className="px-2 pb-2 shrink-0">
        {label && (
          <div className="text-[10px] text-neutral-400 uppercase tracking-wider leading-none mb-1">
            {label}
          </div>
        )}
        <div className="text-xs font-medium text-white truncate leading-tight mb-1">
          {name ?? item.item_uuid}
        </div>
        <div className="flex items-center gap-1">
          <span className="text-[10px] text-white/35 line-through tabular-nums">
            {formatVp(item.base_cost)}
          </span>
          <div className="flex items-center gap-0.5 text-xs text-white/70">
            <VpIcon />
            <span className="tabular-nums">{formatVp(item.discounted_cost)}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

interface ShopWindowProps {
  accountId: number
}

export function ShopWindow({ accountId }: ShopWindowProps) {
  const [storefront, setStorefront] = useState<Storefront | null>(null)
  const [skinMap, setSkinMap] = useState<Record<string, SkinWeapon | null>>({})
  const [itemMap, setItemMap] = useState<Record<string, ItemInfo | null>>({})
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const dailyRemaining = useCountdown(storefront?.daily_remaining_secs ?? null)
  const nightmarketRemaining = useCountdown(storefront?.night_market_remaining_secs ?? null)

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') getCurrentWindow().close()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  useEffect(() => {
    isDemoMode().then((isDemo) => {
      if (isDemo) {
        setStorefront(MOCK_STOREFRONT)
        setSkinMap(MOCK_SKIN_MAP)
        return
      }

      setLoading(true)

      getAccountCookies(accountId)
        .then(async (cookies) => {
          if (!cookies) {
            setError('No session found. Please log in with Riot Client first.')
            return
          }

          const sf = await getShop(accountId, cookies)
          setStorefront(sf)

          // Skin UUIDs: daily offers, night market, and bundle skin items
          const skinUuids = [
            ...sf.daily_offers.map((o) => o.skin_uuid),
            ...(sf.night_market ?? []).map((o) => o.skin_uuid),
            ...(sf.bundles ?? []).flatMap((b) =>
              b.items.filter((i) => i.item_type_id === ITEM_TYPE_SKIN).map((i) => i.item_uuid)
            ),
          ]

          // For bonus bundle items, try all bonus-type DBs regardless of item_type_id.
          // The actual storefront type IDs may differ from our constants, so we let the
          // DB lookups determine the item type rather than relying on type_id matching.
          const bundleItems = (sf.bundles ?? []).flatMap((b) => b.items)
          const bonusBundleUuids = bundleItems
            .filter((i) => i.item_type_id !== ITEM_TYPE_SKIN)
            .map((i) => i.item_uuid)

          const fetches = await Promise.allSettled([
            skinUuids.length > 0 ? getSkinInfoBatch(skinUuids) : Promise.resolve([]),
            bonusBundleUuids.length > 0 ? getBuddyInfoBatch(bonusBundleUuids) : Promise.resolve([]),
            bonusBundleUuids.length > 0 ? getPlayercardInfoBatch(bonusBundleUuids) : Promise.resolve([]),
            bonusBundleUuids.length > 0 ? getSprayInfoBatch(bonusBundleUuids) : Promise.resolve([]),
            bonusBundleUuids.length > 0 ? getFlexInfoBatch(bonusBundleUuids) : Promise.resolve([]),
          ])

          const newSkinMap: Record<string, SkinWeapon | null> = {}
          const newItemMap: Record<string, ItemInfo | null> = {}

          if (fetches[0].status === 'fulfilled') {
            const results = fetches[0].value as (SkinWeapon | null)[]
            skinUuids.forEach((uuid, i) => { newSkinMap[uuid] = results[i] ?? null })
            skinUuids.forEach((uuid, i) => {
              const d = results[i]
              newItemMap[uuid] = d ? { kind: 'skin', data: d } : null
            })
          }

          const buddyResults = fetches[1].status === 'fulfilled'
            ? fetches[1].value as (BuddyItem | null)[]
            : new Array<BuddyItem | null>(bonusBundleUuids.length).fill(null)
          const cardResults = fetches[2].status === 'fulfilled'
            ? fetches[2].value as (PlayercardItem | null)[]
            : new Array<PlayercardItem | null>(bonusBundleUuids.length).fill(null)
          const sprayResults = fetches[3].status === 'fulfilled'
            ? fetches[3].value as (SprayItem | null)[]
            : new Array<SprayItem | null>(bonusBundleUuids.length).fill(null)
          const flexResults = fetches[4].status === 'fulfilled'
            ? fetches[4].value as (FlexItem | null)[]
            : new Array<FlexItem | null>(bonusBundleUuids.length).fill(null)

          bonusBundleUuids.forEach((uuid, i) => {
            const buddy = buddyResults[i]
            const card = cardResults[i]
            const spray = sprayResults[i]
            const flex = flexResults[i]
            if (buddy) newItemMap[uuid] = { kind: 'buddy', data: buddy }
            else if (card) newItemMap[uuid] = { kind: 'playercard', data: card }
            else if (spray) newItemMap[uuid] = { kind: 'spray', data: spray }
            else if (flex) newItemMap[uuid] = { kind: 'flex', data: flex }
            else newItemMap[uuid] = null
          })

          setSkinMap(newSkinMap)
          setItemMap(newItemMap)
        })
        .catch((e) => setError(String(e)))
        .finally(() => setLoading(false))
    })
  }, [accountId])

  const bundles = storefront?.bundles ?? []
  const nightMarket = storefront?.night_market ?? null

  return (
    <div className="min-h-screen bg-neutral-900 text-white flex flex-col">
      <div className="flex-1 overflow-y-auto shop-scrollbar p-6">
        {loading ? (
          <div className="text-sm text-neutral-400 text-center py-8">
            <img src="/refresh-icon.svg" alt="" className="w-5 h-5 animate-spin inline-block" />
          </div>
        ) : error ? (
          <div className="text-sm text-red-400 text-center py-8">{error}</div>
        ) : storefront ? (
          <div className="flex flex-col gap-8">

            <section>
              <SectionHeader label="Daily" countdown={dailyRemaining} />
              <div className="grid grid-cols-4 gap-4">
                {storefront.daily_offers.map((offer) => {
                  const skin = skinMap[offer.skin_uuid] ?? null
                  const hex = tierHex(skin?.tier_color ?? null)
                  return <SkinCard key={offer.skin_uuid} skin={skin} offer={offer} hex={hex} />
                })}
              </div>
            </section>

            {bundles.length > 0 && (
              <section>
                <SectionHeader label="Bundles" />
                <div className="flex flex-col gap-6">
                  {bundles.map((bundle, i) => (
                    <BundleGroup key={i} bundle={bundle} itemMap={itemMap} />
                  ))}
                </div>
              </section>
            )}

            {nightMarket && nightMarket.length > 0 && (
              <section>
                <SectionHeader label="Night Market" countdown={nightmarketRemaining} />
                <div className="grid grid-cols-6 gap-4">
                  {nightMarket.map((offer) => {
                    const skin = skinMap[offer.skin_uuid] ?? null
                    const hex = tierHex(skin?.tier_color ?? null)
                    return (
                      <NightMarketCard
                        key={offer.skin_uuid}
                        skin={skin}
                        offer={offer}
                        hex={hex}
                      />
                    )
                  })}
                </div>
              </section>
            )}

          </div>
        ) : null}
      </div>
    </div>
  )
}


interface NightMarketCardProps {
  skin: SkinWeapon | null
  offer: NightMarketOffer
  hex: string | null
}

function NightMarketCard({ skin, offer, hex }: NightMarketCardProps) {
  return (
    <div
      className="rounded aspect-[3/4] relative overflow-hidden"
      style={cardGradient(hex)}
    >
      <div className="absolute top-2 left-2">
        <span className="text-xs font-bold text-red-400 leading-none tabular-nums">
          -{Math.round(offer.discount_percent)}%
        </span>
      </div>

      <div className="absolute top-2 right-2 flex flex-col items-end gap-0.5">
        <span className="text-[10px] text-white/40 line-through leading-none tabular-nums">
          {formatVp(offer.base_cost)}
        </span>
        <div className="flex items-center gap-0.5 text-xs text-white/80 leading-none">
          <VpIcon />
          <span className="tabular-nums">{formatVp(offer.discount_cost)}</span>
        </div>
      </div>

      <img
        src={skinImageUrl(skin, offer.skin_uuid)}
        alt={skin?.display_name ?? offer.skin_uuid}
        className="w-full h-full object-contain p-3 pb-10"
        loading="lazy"
        onError={(e) => { e.currentTarget.style.display = 'none' }}
      />

      <div className="absolute bottom-0 left-0 right-0 px-2 pb-2 flex items-end justify-between gap-1">
        <span className="text-[11px] font-semibold text-white uppercase tracking-wide leading-tight">
          {skin?.display_name ?? offer.skin_uuid}
        </span>
        {skin?.tier_icon && (
          <img src={skin.tier_icon} alt="" className="w-4 h-4 shrink-0 opacity-80" />
        )}
      </div>
    </div>
  )
}

interface SkinCardProps {
  skin: SkinWeapon | null
  offer: DailyOffer
  hex: string | null
  strikePrice?: number
  discountPercent?: number
  fallbackName?: string
  fallbackIcon?: string | null
}

function SkinCard({ skin, offer, hex, strikePrice, discountPercent, fallbackName, fallbackIcon }: SkinCardProps) {
  const displayName = skin?.display_name ?? fallbackName
  const displayIcon = skin?.display_icon ?? fallbackIcon ?? null
  const hasData = displayName !== undefined

  return (
    <div
      className="rounded aspect-[16/9] relative overflow-hidden"
      style={cardGradient(hex)}
    >
      <div className="absolute top-2 right-3 flex flex-col items-end gap-0.5">
        {strikePrice !== undefined && (
          <span className="text-xs text-white/40 line-through leading-none">
            {formatVp(strikePrice)}
          </span>
        )}
        <div className="flex items-center gap-1 text-sm text-white/80 leading-none">
          <VpIcon />
          <span>{formatVp(offer.vp_cost)}</span>
        </div>
        {discountPercent !== undefined && (
          <span className="text-xs text-green-400 leading-none">
            -{discountPercent}%
          </span>
        )}
      </div>
      {!hasData ? (
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs text-neutral-500 uppercase tracking-widest">No data</span>
        </div>
      ) : (
        <>
          {displayIcon && (
            <img
              src={displayIcon}
              alt={displayName}
              className="w-full h-full object-contain p-4 pb-9"
              loading="lazy"
              onError={(e) => { e.currentTarget.style.display = 'none' }}
            />
          )}
          <div className="absolute bottom-0 left-0 right-0 px-3 pb-2 text-sm font-semibold text-white uppercase tracking-wide leading-tight">
            {displayName}
          </div>
          {skin?.tier_icon && (
            <img src={skin.tier_icon} alt="" className="absolute bottom-2 right-3 w-4 h-4 opacity-80" />
          )}
        </>
      )}
    </div>
  )
}
