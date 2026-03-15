import { useState, useEffect } from 'react'
import type { Account, UpdateAccount } from '../types/account'
import { MODAL_STYLES, RankDropdown, PasswordInput } from './shared'

interface EditAccountModalProps {
  account: Account | null
  onClose: () => void
  onSubmit: (account: UpdateAccount) => Promise<void>
}

export function EditAccountModal({ account, onClose, onSubmit }: EditAccountModalProps) {
  const [riotId, setRiotId] = useState('')
  const [tagline, setTagline] = useState('')
  const [rank, setRank] = useState('Unranked')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (account) {
      setRiotId(account.riot_id)
      setTagline(account.tagline)
      setRank(account.rank ?? 'Unranked')
      setUsername(account.username ?? '')
      setPassword('')
      setError(null)
    }
  }, [account])

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
          <PasswordInput
            value={password}
            onChange={setPassword}
            placeholder="Leave blank to keep current"
          />
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
