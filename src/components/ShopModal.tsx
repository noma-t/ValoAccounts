import { useState, useEffect, useRef } from 'react'
import type { Account } from '../types/account'
import type { RiotCookies, Storefront, SkinWeapon } from '../lib/tauri'
import { getShop, getSkinInfoBatch } from '../lib/tauri'

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

interface ShopModalProps {
  account: Account | null
  cookies: RiotCookies | null
  onClose: () => void
}

const MODAL_STYLES = {
  overlay: 'fixed inset-0 bg-black/70 flex items-center justify-center z-50 overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden py-4',
  dialog: 'bg-neutral-900 border border-neutral-700/70 rounded-lg p-6 pt-4 w-full max-w-lg mx-4 shadow-2xl my-auto max-h-[calc(100vh-2rem)] overflow-y-auto shop-scrollbar',
  header: 'flex items-center justify-between mb-3',
  title: 'text-base font-semibold text-white',
  closeButton: 'text-neutral-400 hover:text-white transition-colors',
  grid: 'grid grid-cols-2 gap-3',
  skinCard: 'rounded aspect-[16/9] relative overflow-hidden',
  skinImage: 'w-full h-full object-contain p-3 pb-7',
  skinName: 'absolute bottom-0 left-0 right-0 px-2 pb-1.5 text-[13px] font-semibold text-white uppercase tracking-wide leading-tight',
  skinCost: 'absolute top-1.5 right-2 flex items-center gap-1 text-[13px] text-white/80',
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

export function ShopModal({ account, cookies, onClose }: ShopModalProps) {
  const [storefront, setStorefront] = useState<Storefront | null>(null)
  const [skinMap, setSkinMap] = useState<Record<string, SkinWeapon | null>>({})
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const dailyRemaining = useCountdown(storefront?.daily_remaining_secs ?? null)

  useEffect(() => {
    if (!account || !cookies) {
      setStorefront(null)
      setSkinMap({})
      setError(null)
      return
    }

    setLoading(true)
    setError(null)
    getShop(account.id, cookies)
      .then(async (sf) => {
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
        } catch (e) {
          log_skin_resolve_error(e)
        }
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false))
  }, [account, cookies])

  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose()
    }
  }

  if (!account) return null

  return (
    <div className={MODAL_STYLES.overlay} onClick={handleOverlayClick}>
      <div className={MODAL_STYLES.dialog}>
        <div className={MODAL_STYLES.header}>
          <div className="flex items-center gap-2">
            <span className={MODAL_STYLES.title}>
              {account.riot_id}{account.tagline ? `#${account.tagline}` : ''}
            </span>
            {dailyRemaining !== null && (
              <span className="text-xs tabular-nums text-neutral-500">
                {formatCountdown(dailyRemaining)}
              </span>
            )}
          </div>
          <button
            type="button"
            className={MODAL_STYLES.closeButton}
            onClick={onClose}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        {!cookies ? (
          <div className="text-sm text-neutral-400 text-center py-8">
            Riot Clientからログインしてください
          </div>
        ) : loading ? (
          <div className="text-sm text-neutral-400 text-center py-8">
            <img src="/refresh-icon.svg" alt="" className="w-5 h-5 animate-spin inline-block" />
          </div>
        ) : error ? (
          <div className="text-sm text-red-400 text-center py-8">
            {error}
          </div>
        ) : storefront ? (
          <div className={MODAL_STYLES.grid}>
                {storefront.daily_offers.map((offer) => {
                  const skin = skinMap[offer.skin_uuid] ?? null
                  const hex = tierHex(skin?.tier_color ?? null)
                  return (
                    <div
                      key={offer.skin_uuid}
                      className={MODAL_STYLES.skinCard}
                      style={cardGradient(hex)}
                    >
                      <div className={MODAL_STYLES.skinCost}>
                        <VpIcon />
                        <span>{formatVp(offer.vp_cost)}</span>
                      </div>
                      <img
                        src={skinImageUrl(skin, offer.skin_uuid)}
                        alt={skin?.display_name ?? offer.skin_uuid}
                        className={MODAL_STYLES.skinImage}
                        loading="lazy"
                      />
                      <div className={MODAL_STYLES.skinName}>
                        {skin?.display_name ?? offer.skin_uuid}
                      </div>
                    </div>
                  )
                })}
          </div>
        ) : null}
      </div>
    </div>
  )
}

function log_skin_resolve_error(e: unknown): void {
  if (typeof window !== 'undefined' && 'console' in window) {
    (window.console as Console).warn('Failed to resolve skin info:', e)
  }
}
