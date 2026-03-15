import type { SkinWeapon, NightMarketOffer } from '../../lib/tauri'
import { cardGradient, skinImageUrl, formatVp, VpIcon } from '../../lib/shop-utils'

interface NightMarketCardProps {
  skin: SkinWeapon | null
  offer: NightMarketOffer
  hex: string | null
}

export function NightMarketCard({ skin, offer, hex }: NightMarketCardProps) {
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
