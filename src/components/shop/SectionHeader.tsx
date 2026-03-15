import { formatCountdown } from '../../lib/shop-utils'

interface SectionHeaderProps {
  label: string
  countdown?: number | null
  onRefresh?: () => void
  refreshDisabled?: boolean
  refreshing?: boolean
}

export function SectionHeader({ label, countdown, onRefresh, refreshDisabled, refreshing }: SectionHeaderProps) {
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
      {onRefresh !== undefined && (
        <button
          onClick={onRefresh}
          disabled={refreshDisabled}
          className="shrink-0 cursor-pointer rounded p-1 text-neutral-400 hover:text-white hover:bg-white/10 active:bg-white/20 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          title={refreshDisabled ? 'RiotClient or Valorant is running' : 'Refresh shop'}
        >
          <img
            src="/refresh-icon.svg"
            alt="Refresh"
            className={`w-3.5 h-3.5${refreshing ? ' animate-spin' : ''}`}
          />
        </button>
      )}
    </div>
  )
}
