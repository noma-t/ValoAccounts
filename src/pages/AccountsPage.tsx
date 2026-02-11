import { useState, useEffect } from 'react'
import { listAccounts, updateAccount, getSettings, switchAccount } from '../lib/tauri'
import { RANK_ICON_MAP } from '../types/account'
import type { Account, UpdateAccount, ValorantRank } from '../types/account'
import { EditAccountModal } from '../components/EditAccountModal'

function rankIconPath(rank: string | null): string {
  const key = (rank ?? 'Unranked') as ValorantRank
  return `/rank_icon/${RANK_ICON_MAP[key] ?? 'unranked'}.png`
}

interface AccountCardProps {
  account: Account
  onCopyRiotId: () => void
  onCopyId: () => void
  onCopyPassword: () => void
  onSettings: () => void
  onSelect: () => void
  isSelected: boolean
  selectDisabled: boolean
  hasApiKey: boolean
}

interface AccountsPageProps {
  refreshToken?: number
  riotClientRunning?: boolean
  valorantRunning?: boolean
  hasApiKey?: boolean
}

const CARD_STYLES = {
  container: "bg-gradient-to-br from-neutral-800 to-neutral-900 rounded-lg flex items-center p-2.5 border border-neutral-700/50 hover:border-neutral-600/50 transition-all duration-200",
  iconWrapper: "relative flex-shrink-0 group/rank overflow-hidden rounded-full",
  icon: "w-7 h-7",
  info: "cursor-default hover:bg-neutral-700/30 rounded transition-colors duration-150 px-2 py-0.5 min-w-0",
  fullIdWrapper: "block truncate",
  riotId: "text-sm font-semibold text-white",
  tag: "text-sm font-medium text-neutral-400",
  actions: "flex items-center gap-1 ml-auto flex-shrink-0",
  secondaryButton: "group relative p-1.5 bg-neutral-800 active:bg-neutral-700 text-neutral-200 text-xs font-medium rounded transition-all duration-200 border border-neutral-600/50 hover:border-neutral-500/50 aspect-square flex items-center justify-center disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-neutral-600/50",
  primaryButton: "group relative px-3 py-1 bg-red-700 hover:bg-red-600 active:bg-red-800 text-white text-xs font-semibold rounded transition-all duration-200 border border-red-700/50",
  buttonContent: "relative z-10",
  buttonOverlay: "absolute inset-0"
}

function AccountCard({ account, onCopyRiotId, onCopyId, onCopyPassword, onSettings, onSelect, isSelected, selectDisabled, hasApiKey }: AccountCardProps) {
  const [isRefreshingRank, setIsRefreshingRank] = useState(false)

  function handleRefreshRank(e: React.MouseEvent) {
    e.stopPropagation()
    if (isRefreshingRank || !hasApiKey) return
    setIsRefreshingRank(true)
    // TODO: API通信実装時はここで呼び出し
  }

  return (
    <div className={CARD_STYLES.container} onClick={selectDisabled ? undefined : onSelect}>
      <div className={CARD_STYLES.iconWrapper}>
        <img
          src={rankIconPath(account.rank)}
          alt={account.rank ?? 'Unranked'}
          className={CARD_STYLES.icon}
        />
        <div
          className={`absolute inset-0 flex items-center justify-center transition-opacity duration-200 bg-black/50 ${
            isRefreshingRank
              ? 'opacity-100'
              : 'opacity-0 group-hover/rank:opacity-100'
          } ${!hasApiKey ? 'cursor-not-allowed' : 'cursor-pointer'}`}
          onClick={handleRefreshRank}
          title={!hasApiKey ? 'API key is not set' : 'Refresh rank'}
        >
          <img
            src="/refresh-icon.svg"
            alt="Refresh rank"
            className={`w-4 h-4 ${isRefreshingRank ? 'animate-spin' : ''} ${!hasApiKey ? 'opacity-40' : ''}`}
          />
        </div>
      </div>

      <div className={CARD_STYLES.info} onClick={(e) => {
        e.stopPropagation()
        onCopyRiotId()
      }}>
        <span className={CARD_STYLES.fullIdWrapper} title="Copy Riot ID">
          <span className={CARD_STYLES.riotId}>{account.riot_id}</span>
          {account.tagline && <span className={CARD_STYLES.tag}>#{account.tagline}</span>}
        </span>
      </div>

      <div className={CARD_STYLES.actions}>
        <button onClick={(e) => {
          e.stopPropagation()
          onCopyId()
        }} className={CARD_STYLES.secondaryButton}
          title="Copy Username"
          disabled={!account.username}>
          <img src="/account-id-icon.svg" alt="ID" className="w-4 h-4 relative z-10" />
          <div className={CARD_STYLES.buttonOverlay}></div>
        </button>

        <button onClick={(e) => {
          e.stopPropagation()
          onCopyPassword()
        }} className={CARD_STYLES.secondaryButton}
          title="Copy Password"
          disabled={!account.has_password}>
          <img src="/password-icon.svg" alt="Password" className="w-4 h-4 relative z-10" />
          <div className={CARD_STYLES.buttonOverlay}></div>
        </button>

        <button onClick={(e) => {
          e.stopPropagation()
          onSettings()
        }} className={CARD_STYLES.secondaryButton}
          title="Open account settings"
          disabled={isRefreshingRank}>
          <img src="/setting-icon.svg" alt="Settings" className="w-4 h-4 relative z-10" />
          <div className={CARD_STYLES.buttonOverlay}></div>
        </button>

        <button
          onClick={(e) => {
            e.stopPropagation()
            if (!selectDisabled) onSelect()
          }}
          disabled={selectDisabled}
          className="flex items-center gap-2 group px-2 py-1 rounded transition-all duration-200 disabled:cursor-not-allowed"
        >
          <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all duration-200 ${
            isSelected
              ? "border-red-500"
              : selectDisabled
                ? "border-neutral-600 bg-transparent"
                : "border-neutral-500 bg-transparent group-hover:border-neutral-400"
          }`}>
            {isSelected && (
              <div className="w-2.5 h-2.5 rounded-full bg-red-500"></div>
            )}
          </div>
        </button>
      </div>
    </div>
  )
}

export function AccountsPage({ refreshToken, riotClientRunning = false, valorantRunning = false, hasApiKey = false }: AccountsPageProps) {
  const [accounts, setAccounts] = useState<Account[]>([])
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null)
  const [editingAccount, setEditingAccount] = useState<Account | null>(null)
  const [isSwitching, setIsSwitching] = useState(false)

  const selectDisabled = riotClientRunning || valorantRunning || isSwitching

  useEffect(() => {
    loadAccounts()
    getSettings()
      .then((settings) => {
        setSelectedAccountId(settings.active_account_id)
      })
      .catch(() => {})
  }, [refreshToken])

  function loadAccounts() {
    listAccounts()
      .then(setAccounts)
      .catch(() => {})
  }

  function handleCopyRiotId(account: Account) {
    navigator.clipboard.writeText(`${account.riot_id}#${account.tagline}`).catch(() => {})
  }

  function handleCopyId(_accountId: number) {
    // TODO: Implement copy ID functionality
  }

  function handleCopyPassword(_accountId: number) {
    // TODO: Implement copy password functionality
  }

  function handleSettings(account: Account) {
    setEditingAccount(account)
  }

  async function handleEditSubmit(data: UpdateAccount) {
    await updateAccount(data)
    loadAccounts()
  }

  async function handleSelect(accountId: number) {
    if (isSwitching) return

    setIsSwitching(true)
    try {
      if (selectedAccountId === accountId) {
        await switchAccount(null)
        setSelectedAccountId(null)
      } else {
        await switchAccount(accountId)
        setSelectedAccountId(accountId)
      }
      loadAccounts()
    } catch (error) {
      console.error('Switch failed:', error)
    } finally {
      setIsSwitching(false)
    }
  }

  return (
    <div data-testid="accounts-page" className="h-full flex flex-col">
      <div className="h-full flex flex-col gap-2 overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden py-3">
        {accounts.map((account) => (
          <AccountCard
            key={account.id}
            account={account}
            onCopyRiotId={() => handleCopyRiotId(account)}
            onCopyId={() => handleCopyId(account.id)}
            onCopyPassword={() => handleCopyPassword(account.id)}
            onSettings={() => handleSettings(account)}
            onSelect={() => handleSelect(account.id)}
            isSelected={selectedAccountId === account.id}
            selectDisabled={selectDisabled}
            hasApiKey={hasApiKey}
          />
        ))}
      </div>
      <EditAccountModal
        account={editingAccount}
        onClose={() => setEditingAccount(null)}
        onSubmit={handleEditSubmit}
      />
    </div>
  )
}
