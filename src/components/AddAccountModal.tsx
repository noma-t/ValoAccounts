import { useState, useEffect } from 'react'
import type { CreateAccount } from '../types/account'
import { getSettings } from '../lib/tauri'
import { MODAL_STYLES, RankDropdown, PasswordInput } from './shared'

interface AddAccountModalProps {
  isOpen: boolean
  hasApiKey: boolean
  isCurrentDataAvailable: boolean
  riotClientRunning: boolean
  valorantRunning: boolean
  onClose: () => void
  onSubmit: (account: CreateAccount) => Promise<void>
}

export function AddAccountModal({
  isOpen,
  hasApiKey: _hasApiKey,
  isCurrentDataAvailable,
  riotClientRunning,
  valorantRunning,
  onClose,
  onSubmit,
}: AddAccountModalProps) {
  const [riotId, setRiotId] = useState('')
  const [tagline, setTagline] = useState('')
  const [rank, setRank] = useState('Unranked')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [useCurrentData, setUseCurrentData] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [activeAccountId, setActiveAccountId] = useState<number | null>(null)

  useEffect(() => {
    if (!isOpen) {
      setRiotId('')
      setTagline('')
      setRank('Unranked')
      setUsername('')
      setPassword('')
      setUseCurrentData(false)
      setIsSubmitting(false)
      setError(null)
    } else {
      getSettings()
        .then((settings) => setActiveAccountId(settings.active_account_id))
        .catch(() => {})
    }
  }, [isOpen])

  useEffect(() => {
    if ((!isCurrentDataAvailable || riotClientRunning || valorantRunning) && useCurrentData) {
      setUseCurrentData(false)
    }
  }, [isCurrentDataAvailable, riotClientRunning, valorantRunning, useCurrentData])

  async function handleSubmit() {
    if (!riotId.trim()) {
      setError('Riot ID is required')
      return
    }

    setIsSubmitting(true)
    setError(null)

    try {
      await onSubmit({
        riot_id: riotId.trim(),
        tagline: tagline.trim(),
        username: username.trim() || null,
        password: password || null,
        rank: rank,
        use_current_data: useCurrentData,
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

  if (!isOpen) return null

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
            <RankDropdown rank={rank} onRankChange={setRank} />
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
          <PasswordInput value={password} onChange={setPassword} />
        </div>

        <div className={MODAL_STYLES.divider} />

        {error && (
          <p className="text-xs text-red-400 mb-3">{error}</p>
        )}

        <div className="flex items-center justify-between">
          <div className="flex gap-3">
            <label
              className={isCurrentDataAvailable && activeAccountId === null && !riotClientRunning && !valorantRunning ? MODAL_STYLES.radioLabel : MODAL_STYLES.radioLabelDisabled}
              title={
                riotClientRunning
                  ? "Riot Client is running. Close it first."
                  : valorantRunning
                    ? "Valorant is running. Close it first."
                    : activeAccountId !== null
                      ? "Current data is already linked to an account"
                      : !isCurrentDataAvailable
                        ? "Current data not available"
                        : ""
              }
            >
              <input
                type="radio"
                name="data-mode"
                checked={useCurrentData}
                onChange={() => setUseCurrentData(true)}
                disabled={!isCurrentDataAvailable || activeAccountId !== null || riotClientRunning || valorantRunning}
              />
              <span className={MODAL_STYLES.radioText}>Current</span>
            </label>
            <label className={MODAL_STYLES.radioLabel}>
              <input
                type="radio"
                name="data-mode"
                checked={!useCurrentData}
                onChange={() => setUseCurrentData(false)}
              />
              <span className={MODAL_STYLES.radioText}>New</span>
            </label>
          </div>
          <div className="flex gap-2">
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
              {isSubmitting ? 'Adding...' : 'Add Account'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
