import { useState, useEffect, useRef } from 'react'
import { VALORANT_RANKS, RANK_ICON_MAP } from '../types/account'
import type { Account, UpdateAccount, ValorantRank } from '../types/account'

interface EditAccountModalProps {
  account: Account | null
  onClose: () => void
  onSubmit: (account: UpdateAccount) => Promise<void>
}

const MODAL_STYLES = {
  overlay: 'fixed inset-0 bg-black/70 flex items-center justify-center z-50',
  dialog: 'bg-neutral-900 border border-neutral-700/70 rounded-lg p-5 w-full max-w-sm mx-4 shadow-2xl',
  fieldGroup: 'mb-3',
  label: 'block text-xs font-medium text-neutral-400 mb-1',
  input: 'w-full bg-neutral-800 border border-neutral-700/50 rounded px-2.5 py-1.5 text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-neutral-500 transition-colors',
  actions: 'flex justify-end gap-2 mt-4',
  cancelButton: 'px-3 py-1.5 bg-neutral-800 hover:bg-neutral-700 active:bg-neutral-750 border border-neutral-700/50 text-neutral-300 text-sm rounded transition-colors',
  submitButton: 'px-3 py-1.5 bg-red-700 hover:bg-red-600 active:bg-red-800 text-white text-sm font-medium rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed',
}

export function EditAccountModal({ account, onClose, onSubmit }: EditAccountModalProps) {
  const [riotId, setRiotId] = useState('')
  const [tagline, setTagline] = useState('')
  const [rank, setRank] = useState('Unranked')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showRankDropdown, setShowRankDropdown] = useState(false)
  const rankDropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (account) {
      setRiotId(account.riot_id)
      setTagline(account.tagline)
      setRank(account.rank ?? 'Unranked')
      setUsername(account.username ?? '')
      setPassword('')
      setShowPassword(false)
      setShowRankDropdown(false)
      setError(null)
    }
  }, [account])

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (rankDropdownRef.current && !rankDropdownRef.current.contains(e.target as Node)) {
        setShowRankDropdown(false)
      }
    }
    if (showRankDropdown) {
      document.addEventListener('mousedown', handleClickOutside)
    }
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [showRankDropdown])

  async function handleSubmit() {
    if (!account) return
    if (!riotId.trim()) {
      setError('Riot ID is required')
      return
    }

    setIsSubmitting(true)
    setError(null)

    try {
      await onSubmit({
        id: account.id,
        riot_id: riotId.trim(),
        tagline: tagline.trim(),
        username: username.trim() || null,
        password: password || null,
        rank: rank,
      })
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsSubmitting(false)
    }
  }

  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose()
    }
  }

  if (!account) return null

  return (
    <div className={MODAL_STYLES.overlay} onClick={handleOverlayClick}>
      <div className={MODAL_STYLES.dialog}>
        <div className={MODAL_STYLES.fieldGroup}>
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <label className={MODAL_STYLES.label}>Riot ID</label>
              <div className="flex items-center gap-1.5">
                <input
                  type="text"
                  className={MODAL_STYLES.input}
                  placeholder="Game name (required)"
                  value={riotId}
                  onChange={(e) => setRiotId(e.target.value)}
                />
                <span className="text-neutral-500 font-medium select-none">#</span>
                <input
                  type="text"
                  className="w-24 bg-neutral-800 border border-neutral-700/50 rounded px-2.5 py-1.5 text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-neutral-500 transition-colors"
                  placeholder="Tag"
                  value={tagline}
                  onChange={(e) => setTagline(e.target.value)}
                />
              </div>
            </div>
            <div className="relative" ref={rankDropdownRef}>
              <label className={MODAL_STYLES.label}>Rank</label>
              <button
                type="button"
                className="flex items-center justify-center p-1.5 bg-neutral-800 border border-neutral-700/50 rounded hover:border-neutral-500 transition-colors"
                onClick={() => setShowRankDropdown(!showRankDropdown)}
                title={rank}
              >
                <img
                  src={`/rank_icon/${RANK_ICON_MAP[rank as ValorantRank]}.png`}
                  alt={rank}
                  className="w-5 h-5 object-contain"
                />
              </button>
              {showRankDropdown && (
                <div className="absolute right-0 top-full mt-1 w-44 bg-neutral-900 border border-neutral-700/70 rounded shadow-2xl z-10 max-h-52 overflow-y-auto">
                  {VALORANT_RANKS.map((r) => (
                    <button
                      key={r}
                      type="button"
                      className={`flex items-center gap-2 w-full px-2.5 py-1.5 text-sm text-left transition-colors ${rank === r ? 'bg-neutral-700 text-white' : 'text-neutral-300 hover:bg-neutral-800'}`}
                      onClick={() => { setRank(r); setShowRankDropdown(false) }}
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
          </div>
        </div>

        <div className={MODAL_STYLES.fieldGroup}>
          <label className={MODAL_STYLES.label}>Username</label>
          <input
            type="text"
            className={MODAL_STYLES.input}
            placeholder="Username / Email"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />
        </div>

        <div className={MODAL_STYLES.fieldGroup}>
          <label className={MODAL_STYLES.label}>Password</label>
          <div className="relative">
            <input
              type={showPassword ? 'text' : 'password'}
              className="w-full bg-neutral-800 border border-neutral-700/50 rounded px-2.5 py-1.5 pr-8 text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-neutral-500 transition-colors"
              placeholder="Leave blank to keep current"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <button
              type="button"
              className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-500 hover:text-neutral-300 transition-colors"
              onClick={() => setShowPassword(!showPassword)}
              title={showPassword ? 'Hide password' : 'Show password'}
            >
              {showPassword ? (
                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94" />
                  <path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19" />
                  <line x1="1" y1="1" x2="23" y2="23" />
                </svg>
              ) : (
                <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              )}
            </button>
          </div>
        </div>

        {error && (
          <p className="text-xs text-red-400 mb-3">{error}</p>
        )}

        <div className={MODAL_STYLES.actions}>
          <button
            type="button"
            className={MODAL_STYLES.cancelButton}
            onClick={onClose}
          >
            Cancel
          </button>
          <button
            type="button"
            className={MODAL_STYLES.submitButton}
            onClick={handleSubmit}
            disabled={isSubmitting || !riotId.trim()}
          >
            {isSubmitting ? 'Saving...' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  )
}
