import type { SkinWeapon, DailyOffer } from '../../lib/tauri'
import { cardGradient, formatVp, VpIcon } from '../../lib/shop-utils'

interface SkinCardProps {
  skin: SkinWeapon | null
  offer: DailyOffer
  hex: string | null
  strikePrice?: number
  discountPercent?: number
  fallbackName?: string
  fallbackIcon?: string | null
}

export function SkinCard({ skin, offer, hex, strikePrice, discountPercent, fallbackName, fallbackIcon }: SkinCardProps) {
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
