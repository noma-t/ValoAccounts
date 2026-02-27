import { useState, useEffect, useRef } from 'react'
import { getAccountCookies, getShop, getSkinInfoBatch, isDemoMode } from '../lib/tauri'
import type { Storefront, SkinWeapon, DailyOffer } from '../lib/tauri'
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
  return (
    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M12 2L3 7v6c0 5.25 3.75 10.14 9 11.25C17.25 23.14 21 18.25 21 13V7l-9-5z" fill="currentColor" opacity="0.7" />
    </svg>
  )
}

function skinImageUrl(skin: SkinWeapon | null, levelUuid: string): string {
  if (skin?.display_icon) return skin.display_icon
  return `https://media.valorant-api.com/weaponskinlevels/${levelUuid}/displayicon.png`
}

function formatVp(vp: number): string {
  return vp.toLocaleString()
}

const MOCK_OFFERS: DailyOffer[] = [
  { skin_uuid: 'mock-a', vp_cost: 1775 },
  { skin_uuid: 'mock-b', vp_cost: 2175 },
  { skin_uuid: 'mock-c', vp_cost: 3550 },
  { skin_uuid: 'mock-d', vp_cost: 1275 },
]

const MOCK_SKIN_MAP: Record<string, SkinWeapon> = {
  'mock-a': { uuid: 'mock-a', display_name: 'DEMO Phantom', display_icon: null, tier_color: 'FF4655', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-b': { uuid: 'mock-b', display_name: 'DEMO Vandal', display_icon: null, tier_color: '009BDE', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-c': { uuid: 'mock-c', display_name: 'DEMO Operator', display_icon: null, tier_color: 'F5A623', tier_uuid: null, tier_rank: null, tier_icon: null },
  'mock-d': { uuid: 'mock-d', display_name: 'DEMO Knife', display_icon: null, tier_color: 'BD3944', tier_uuid: null, tier_rank: null, tier_icon: null },
}

const MOCK_STOREFRONT: Storefront = {
  daily_offers: MOCK_OFFERS,
  daily_remaining_secs: 3600 * 8,
  night_market: null,
  night_market_remaining_secs: null,
}

interface ShopWindowProps {
  accountId: number
}

export function ShopWindow({ accountId }: ShopWindowProps) {
  const [storefront, setStorefront] = useState<Storefront | null>(null)
  const [skinMap, setSkinMap] = useState<Record<string, SkinWeapon | null>>({})
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const dailyRemaining = useCountdown(storefront?.daily_remaining_secs ?? null)

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

          const allUuids = [
            ...sf.daily_offers.map((o) => o.skin_uuid),
            ...(sf.night_market ?? []).map((o) => o.skin_uuid),
          ]

          if (allUuids.length === 0) return

          try {
            const results = await getSkinInfoBatch(allUuids)
            const map: Record<string, SkinWeapon | null> = {}
            for (let i = 0; i < allUuids.length; i++) {
              map[allUuids[i]] = results[i] ?? null
            }
            setSkinMap(map)
          } catch {
            // skin info load failure is non-fatal
          }
        })
        .catch((e) => setError(String(e)))
        .finally(() => setLoading(false))
    })
  }, [accountId])

  return (
    <div className="min-h-screen bg-neutral-900 text-white flex flex-col">
      {dailyRemaining !== null && (
        <div className="px-4 py-2 border-b border-neutral-700/50">
          <span className="text-xs tabular-nums text-neutral-500">
            {formatCountdown(dailyRemaining)}
          </span>
        </div>
      )}

      <div className="flex-1 overflow-y-auto shop-scrollbar p-4">
        {loading ? (
          <div className="text-sm text-neutral-400 text-center py-8">
            <img src="/refresh-icon.svg" alt="" className="w-5 h-5 animate-spin inline-block" />
          </div>
        ) : error ? (
          <div className="text-sm text-red-400 text-center py-8">{error}</div>
        ) : storefront ? (
          <div className="flex flex-col gap-4">
            <div className="grid grid-cols-2 gap-3">
              {storefront.daily_offers.map((offer) => {
                const skin = skinMap[offer.skin_uuid] ?? null
                const hex = tierHex(skin?.tier_color ?? null)
                return (
                  <SkinCard
                    key={offer.skin_uuid}
                    skin={skin}
                    offer={offer}
                    hex={hex}
                  />
                )
              })}
            </div>

            {storefront.night_market && storefront.night_market.length > 0 && (
              <div>
                <div className="text-xs font-semibold text-neutral-400 uppercase tracking-wide mb-2">
                  Night Market
                </div>
                <div className="grid grid-cols-2 gap-3">
                  {storefront.night_market.map((offer) => {
                    const skin = skinMap[offer.skin_uuid] ?? null
                    const hex = tierHex(skin?.tier_color ?? null)
                    return (
                      <SkinCard
                        key={offer.skin_uuid}
                        skin={skin}
                        offer={{ skin_uuid: offer.skin_uuid, vp_cost: offer.discount_cost }}
                        hex={hex}
                        strikePrice={offer.base_cost}
                        discountPercent={offer.discount_percent}
                      />
                    )
                  })}
                </div>
              </div>
            )}
          </div>
        ) : null}
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
}

function SkinCard({ skin, offer, hex, strikePrice, discountPercent }: SkinCardProps) {
  return (
    <div
      className="rounded aspect-[16/9] relative overflow-hidden"
      style={cardGradient(hex)}
    >
      <div className="absolute top-1.5 right-2 flex flex-col items-end gap-0.5">
        {strikePrice !== undefined && (
          <span className="text-[11px] text-white/40 line-through leading-none">
            {formatVp(strikePrice)}
          </span>
        )}
        <div className="flex items-center gap-1 text-[13px] text-white/80">
          <VpIcon />
          <span>{formatVp(offer.vp_cost)}</span>
        </div>
        {discountPercent !== undefined && (
          <span className="text-[10px] text-green-400 leading-none">
            -{discountPercent}%
          </span>
        )}
      </div>
      <img
        src={skinImageUrl(skin, offer.skin_uuid)}
        alt={skin?.display_name ?? offer.skin_uuid}
        className="w-full h-full object-contain p-3 pb-7"
        loading="lazy"
        onError={(e) => { e.currentTarget.style.display = 'none' }}
      />
      <div className="absolute bottom-0 left-0 right-0 px-2 pb-1.5 text-[13px] font-semibold text-white uppercase tracking-wide leading-tight">
        {skin?.display_name ?? offer.skin_uuid}
      </div>
    </div>
  )
}
