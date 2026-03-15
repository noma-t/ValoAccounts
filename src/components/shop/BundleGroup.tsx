import { useCountdown } from '../../hooks/useCountdown'
import { ITEM_TYPE_SKIN } from '../../lib/tauri'
import type { Bundle } from '../../lib/tauri'
import { formatCountdown, formatVp, tierHex, VpIcon } from '../../lib/shop-utils'
import type { ItemInfo } from '../../lib/shop-utils'
import { SkinCard } from './SkinCard'

interface BundleGroupProps {
  bundle: Bundle
  itemMap: Record<string, ItemInfo | null>
}

export function BundleGroup({ bundle, itemMap }: BundleGroupProps) {
  const remaining = useCountdown(bundle.bundle_remaining_secs)

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
  } else if (info?.kind === 'title') {
    icon = '/valo-icon.svg'
    name = info.data.display_name
    label = 'Title'
  }

  return (
    <div className="w-[276px] h-[155px] shrink-0 rounded overflow-hidden bg-neutral-800/60 relative">
      {icon ? (
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

      <div className="absolute top-2 right-2 flex flex-col items-end gap-0.5">
        {item.base_cost !== item.discounted_cost && (
          <div className="flex items-center gap-1 leading-none">
            <span className="text-xs text-white/40 line-through tabular-nums">
              {formatVp(item.base_cost)}
            </span>
            <span className="text-xs font-semibold text-green-400 leading-none">
              -{Math.round(item.discount_percent)}%
            </span>
          </div>
        )}
        <div className="flex items-baseline gap-0.5 text-sm text-white leading-none">
          <VpIcon />
          <span className="tabular-nums">{formatVp(item.discounted_cost)}</span>
        </div>
      </div>

      <div className="absolute bottom-0 left-0 right-0 px-2 pb-2">
        {label && (
          <div className="text-xs text-neutral-400 uppercase tracking-wider leading-none mb-0.5">
            {label}
          </div>
        )}
        <div className="text-sm font-medium text-white truncate leading-tight">
          {name ?? item.item_uuid}
        </div>
      </div>
    </div>
  )
}
