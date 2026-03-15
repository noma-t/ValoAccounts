import { useState, useEffect, useRef } from 'react'
import { VALORANT_RANKS, RANK_ICON_MAP } from '../../types/account'
import type { ValorantRank } from '../../types/account'
import { MODAL_STYLES } from './ModalStyles'

interface RankDropdownProps {
  rank: string
  onRankChange: (rank: string) => void
}

export function RankDropdown({ rank, onRankChange }: RankDropdownProps) {
  const [showDropdown, setShowDropdown] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowDropdown(false)
      }
    }
    if (showDropdown) {
      document.addEventListener('mousedown', handleClickOutside)
    }
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [showDropdown])

  return (
    <div className="relative" ref={dropdownRef}>
      <label className={MODAL_STYLES.label}>Rank</label>
      <button
        type="button"
        className="flex items-center justify-center p-1.5 bg-neutral-800 border border-neutral-700/50 rounded hover:border-neutral-500 transition-colors"
        onClick={() => setShowDropdown(!showDropdown)}
        title={rank}
      >
        <img
          src={`/rank_icon/${RANK_ICON_MAP[rank as ValorantRank]}.png`}
          alt={rank}
          className="w-5 h-5 object-contain"
        />
      </button>
      {showDropdown && (
        <div className="absolute right-0 top-full mt-1 w-44 bg-neutral-900 border border-neutral-700/70 rounded shadow-2xl z-10 max-h-52 overflow-y-auto">
          {VALORANT_RANKS.map((r) => (
            <button
              key={r}
              type="button"
              className={`flex items-center gap-2 w-full px-2.5 py-1.5 text-sm text-left transition-colors ${rank === r ? 'bg-neutral-700 text-white' : 'text-neutral-300 hover:bg-neutral-800'}`}
              onClick={() => { onRankChange(r); setShowDropdown(false) }}
            >
              <img
                src={`/rank_icon/${RANK_ICON_MAP[r]}.png`}
                alt={r}
                className="w-5 h-5 object-contain"
              />
              <span>{r}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
