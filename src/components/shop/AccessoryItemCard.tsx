import type { AccessoryOffer } from '../../lib/tauri'
import { formatVp } from '../../lib/shop-utils'
import type { ItemInfo } from '../../lib/shop-utils'

interface AccessoryItemCardProps {
  offer: AccessoryOffer
  info: ItemInfo | null
}

export function AccessoryItemCard({ offer, info }: AccessoryItemCardProps) {
  let icon: string | null = null
  let name: string | null = null
  let label: string | null = null
  let titleText: string | null = null

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
  } else if (info?.kind === 'title') {
    name = info.data.display_name
    titleText = info.data.title_text ?? null
    label = 'Title'
  }

  return (
    <div className="rounded overflow-hidden bg-neutral-800/60 relative h-[155px]">
      {titleText !== null ? (
        <div className="absolute inset-0 bottom-10 flex items-center justify-center">
          <img src="/valo-icon.svg" alt="" className="w-14 h-14 shrink-0 mt-6" />
        </div>
      ) : icon ? (
        <img
          src={icon}
          alt={name ?? ''}
          className="absolute inset-0 w-full h-full object-contain p-3 pb-10"
          loading="lazy"
          onError={(e) => { e.currentTarget.style.display = 'none' }}
        />
      ) : (
        <div className="absolute inset-3 rounded bg-neutral-700/50" />
      )}

      <div className="absolute top-2 right-2 flex items-baseline gap-0.5 text-sm text-green-300 leading-none">
        <span className="text-sm font-bold text-green-400">K</span>
        <span className="tabular-nums">{formatVp(offer.kc_cost)}</span>
      </div>

      <div className="absolute bottom-0 left-0 right-0 px-2 pb-2">
        {label && (
          <div className="text-xs text-neutral-400 uppercase tracking-wider leading-none mb-0.5">
            {label}
          </div>
        )}
        <div className="text-sm font-medium text-white truncate leading-tight">
          {name ?? offer.item_uuid}
        </div>
      </div>
    </div>
  )
}
