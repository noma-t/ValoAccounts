import { useState, useEffect } from 'react'
import { listAccounts, updateAccount, getSettings, switchAccount, copyAccountPassword, openTrackerProfile, getAccountCookies } from '../lib/tauri'
import type { RiotCookies } from '../lib/tauri'
import { RANK_ICON_MAP } from '../types/account'
import type { Account, UpdateAccount, ValorantRank } from '../types/account'
import { EditAccountModal } from '../components/EditAccountModal'
import { ShopModal } from '../components/ShopModal'
import { useToast } from '../components/Toast'

const TIER_ID_TO_RANK: Record<number, string | null> = {
  0: null,
  1: 'Unknown', 2: 'Unknown',
  3: 'Iron 1', 4: 'Iron 2', 5: 'Iron 3',
  6: 'Bronze 1', 7: 'Bronze 2', 8: 'Bronze 3',
  9: 'Silver 1', 10: 'Silver 2', 11: 'Silver 3',
  12: 'Gold 1', 13: 'Gold 2', 14: 'Gold 3',
  15: 'Platinum 1', 16: 'Platinum 2', 17: 'Platinum 3',
  18: 'Diamond 1', 19: 'Diamond 2', 20: 'Diamond 3',
  21: 'Ascendant 1', 22: 'Ascendant 2', 23: 'Ascendant 3',
  24: 'Immortal 1', 25: 'Immortal 2', 26: 'Immortal 3',
  27: 'Radiant',
}

function rankIconPath(rank: string | null): string {
  if (rank === 'Unknown') return '/rank_icon/error.png'
  const key = (rank ?? 'Unranked') as ValorantRank
  return `/rank_icon/${RANK_ICON_MAP[key] ?? 'unranked'}.png`
}

interface AccountCardProps {
  account: Account
  onCopyRiotId: () => void
  onCopyId: () => void
  onCopyPassword: () => void
  onOpenTracker: () => void
  onShop: () => void
  onSettings: () => void
  onSelect: () => void
  onRefreshRank: () => Promise<void>
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

function AccountCard({ account, onCopyRiotId, onCopyId, onCopyPassword, onOpenTracker, onShop, onSettings, onSelect, onRefreshRank, isSelected, selectDisabled, hasApiKey }: AccountCardProps) {
  const [isRefreshingRank, setIsRefreshingRank] = useState(false)

  const canRefresh = hasApiKey && !!account.riot_id && !!account.tagline

  const refreshTitle = !hasApiKey
    ? 'API key is not set'
    : !account.riot_id || !account.tagline
      ? 'Riot ID and tagline are required'
      : 'Refresh rank'

  async function handleRefreshRank(e: React.MouseEvent) {
    e.stopPropagation()
    if (isRefreshingRank || !canRefresh) return
    setIsRefreshingRank(true)
    try {
      await onRefreshRank()
    } finally {
      setIsRefreshingRank(false)
    }
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
          } ${!canRefresh ? 'cursor-not-allowed' : 'cursor-pointer'}`}
          onClick={handleRefreshRank}
          title={refreshTitle}
        >
          <img
            src="/refresh-icon.svg"
            alt="Refresh rank"
            className={`w-4 h-4 ${isRefreshingRank ? 'animate-spin' : ''} ${!canRefresh ? 'opacity-40' : ''}`}
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
          onOpenTracker()
        }} className={CARD_STYLES.secondaryButton}
          style={{ padding: '4px' }}
          title="Open Tracker"
          disabled={!account.riot_id || !account.tagline}>
          <img src="/tracker.svg" alt="Tracker" className="w-5 h-5 relative z-10" />
          <div className={CARD_STYLES.buttonOverlay}></div>
        </button>

        <button onClick={(e) => {
          e.stopPropagation()
          onShop()
        }} className={CARD_STYLES.secondaryButton}
          title="Shop"
          style={{ padding: '5px' }}
          disabled={selectDisabled}
        >
          <img src="/shop-cart-icon.svg" alt="Shop" className="w-4.5 h-4.5 relative z-10" />
          <div className={CARD_STYLES.buttonOverlay}></div>
        </button>

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
  const { toast } = useToast()
  const [accounts, setAccounts] = useState<Account[]>([])
  const [selectedAccountId, setSelectedAccountId] = useState<number | null>(null)
  const [editingAccount, setEditingAccount] = useState<Account | null>(null)
  const [shopAccount, setShopAccount] = useState<Account | null>(null)
  const [shopCookies, setShopCookies] = useState<RiotCookies | null>(null)
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
    const text = `${account.riot_id}#${account.tagline}`
    navigator.clipboard.writeText(text)
      .then(() => toast('success', 'Copied Riot ID'))
      .catch(() => toast('error', 'Failed to copy'))
  }

  function handleCopyId(username: string | null) {
    if (!username) return
    navigator.clipboard.writeText(username)
      .then(() => toast('success', 'Copied username'))
      .catch(() => toast('error', 'Failed to copy'))
  }

  async function handleCopyPassword(accountId: number) {
    try {
      await copyAccountPassword(accountId)
      toast('success', 'Copied password')
    } catch (error) {
      console.error('[handleCopyPassword] accountId:', accountId, 'error:', error)
      toast('error', 'Failed to copy password')
    }
  }

  function handleSettings(account: Account) {
    setEditingAccount(account)
  }

  async function handleEditSubmit(data: UpdateAccount) {
    await updateAccount(data)
    loadAccounts()
  }

  function handleOpenTracker(account: Account) {
    if (!account.riot_id || !account.tagline) return
    openTrackerProfile(account.riot_id, account.tagline).catch(() => {})
  }

  async function handleOpenShop(account: Account) {
    try {
      const cookies = await getAccountCookies(account.id)
      setShopCookies(cookies)
    } catch {
      setShopCookies(null)
    }
    setShopAccount(account)
  }

  async function handleRefreshRank(account: Account) {
    try {
      const settings = await getSettings()
      const region = settings.region ?? 'ap'
      const apiKey = settings.henrikdev_api_key
      if (!apiKey) return

      const response = await fetch(
        `https://api.henrikdev.xyz/valorant/v3/mmr/${region}/pc/${encodeURIComponent(account.riot_id)}/${encodeURIComponent(account.tagline)}`,
        { headers: { 'Authorization': apiKey } }
      )

      if (!response.ok) {
        switch (response.status) {
          case 400: toast('warning', 'Invalid request'); return
          case 403: toast('error', 'API access forbidden'); return
          case 404: toast('warning', 'Player not found'); return
          case 408: toast('warning', 'Request timed out'); return
          case 429: toast('warning', 'Rate limit reached, try again later'); return
          case 503: toast('error', 'Riot API is unavailable'); return
          default: toast('error', `Rank fetch failed (${response.status})`); return
        }
      }

      const json = await response.json()
      const tierId: number | undefined = json?.data?.current?.tier?.id

      if (tierId === undefined) {
        toast('error', 'Unexpected API response')
        return
      }

      const rank = TIER_ID_TO_RANK[tierId] ?? null

      await updateAccount({
        id: account.id,
        riot_id: account.riot_id,
        tagline: account.tagline,
        username: account.username,
        password: null,
        rank,
      })

      loadAccounts()
    } catch {
      toast('error', 'Failed to fetch rank')
    }
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
            onCopyId={() => handleCopyId(account.username)}
            onCopyPassword={() => handleCopyPassword(account.id)}
            onOpenTracker={() => handleOpenTracker(account)}
            onShop={() => handleOpenShop(account)}
            onSettings={() => handleSettings(account)}
            onSelect={() => handleSelect(account.id)}
            onRefreshRank={() => handleRefreshRank(account)}
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
      <ShopModal
        account={shopAccount}
        cookies={shopCookies}
        onClose={() => setShopAccount(null)}
      />
    </div>
  )
}
