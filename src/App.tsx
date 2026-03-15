import { useState, useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { Layout } from './components/Layout'
import { AddAccountModal } from './components/AddAccountModal'
import { ToastProvider } from './components/Toast'
import { AccountsPage } from './pages/AccountsPage'
import { SettingsPage } from './pages/SettingsPage'
import { getSettings, markLaunched, addAccount, checkCurrentDataAvailable, getRiotClientStatus, getValorantStatus, killRiotClient, launchRiotClient } from './lib/tauri'
import type { NavigationItem } from './types/layout'
import type { CreateAccount } from './types/account'
import './App.css'

const navigationItems: NavigationItem[] = [
  { id: 'accounts', label: 'Accounts', icon: '' },
  { id: 'settings', label: 'Settings', icon: '' },
]

function App() {
  const [activePage, setActivePage] = useState<string | null>(null)
  const [hasApiKey, setHasApiKey] = useState(false)
  const [riotClientRunning, setRiotClientRunning] = useState(true)
  const [valorantRunning, setValorantRunning] = useState(true)
  const [isAddAccountModalOpen, setIsAddAccountModalOpen] = useState(false)
  const [isCurrentDataAvailable, setIsCurrentDataAvailable] = useState(false)
  const [accountsRefreshToken, setAccountsRefreshToken] = useState(0)

  useEffect(() => {
    const initializeApp = async () => {
      try {
        const settings = await getSettings()
        setHasApiKey(!!settings.henrikdev_api_key)
        if (!settings.launched) {
          await markLaunched()
          setActivePage('settings')
        } else {
          setActivePage('accounts')
        }
      } catch {
        setHasApiKey(false)
        setActivePage('settings')
      }
    }
    void initializeApp()
  }, [])

  const refreshSettings = useCallback(async () => {
    try {
      const settings = await getSettings()
      setHasApiKey(!!settings.henrikdev_api_key)
    } catch {
      setHasApiKey(false)
    }
  }, [])

  useEffect(() => {
    let cleanup: (() => void) | null = null

    const setupListeners = async () => {
      const [unlisten, unlistenValorant] = await Promise.all([
        listen<boolean>('riot-client-status', (event) => {
          setRiotClientRunning(event.payload)
        }),
        listen<boolean>('valorant-status', (event) => {
          setValorantRunning(event.payload)
        }),
      ])

      cleanup = () => {
        unlisten()
        unlistenValorant()
      }

      const [riotStatus, valorantStatus] = await Promise.all([
        getRiotClientStatus().catch(() => false),
        getValorantStatus().catch(() => false),
      ])
      setRiotClientRunning(riotStatus)
      setValorantRunning(valorantStatus)
    }

    void setupListeners()

    return () => {
      cleanup?.()
    }
  }, [])

  async function handleOpenAddAccount() {
    try {
      const available = await checkCurrentDataAvailable()
      setIsCurrentDataAvailable(available)
    } catch {
      setIsCurrentDataAvailable(false)
    }
    setIsAddAccountModalOpen(true)
  }

  async function handleKillRiotClient() {
    try {
      await killRiotClient()
    } catch {
      // Kill failed silently
    }
  }

  async function handleLaunchRiotClient() {
    try {
      await launchRiotClient()
    } catch {
      // Launch failed silently
    }
  }

  async function handleAddAccount(account: CreateAccount) {
    await addAccount(account)
    setAccountsRefreshToken((t) => t + 1)
  }

  const handleSettingsUpdated = () => {
    void refreshSettings()
  }

  const renderContent = () => {
    if (activePage === null) {
      return <div className="flex items-center justify-center h-full text-neutral-400">Loading...</div>
    }

    switch (activePage) {
      case 'accounts':
        return (
          <AccountsPage
            refreshToken={accountsRefreshToken}
            riotClientRunning={riotClientRunning}
            valorantRunning={valorantRunning}
            hasApiKey={hasApiKey}
          />
        )
      case 'settings':
        return <SettingsPage onSettingsUpdated={handleSettingsUpdated} />
      default:
        return null
    }
  }

  return (
    <ToastProvider>
      <Layout
        navigationItems={navigationItems}
        activeItemId={activePage ?? 'accounts'}
        onNavigate={setActivePage}
        onAddAccount={handleOpenAddAccount}
        riotClientRunning={riotClientRunning}
        valorantRunning={valorantRunning}
        onKillRiotClient={handleKillRiotClient}
        onLaunchRiotClient={handleLaunchRiotClient}
      >
        {renderContent()}
      </Layout>
      <AddAccountModal
        isOpen={isAddAccountModalOpen}
        hasApiKey={hasApiKey}
        isCurrentDataAvailable={isCurrentDataAvailable}
        riotClientRunning={riotClientRunning}
        valorantRunning={valorantRunning}
        onClose={() => setIsAddAccountModalOpen(false)}
        onSubmit={handleAddAccount}
      />
    </ToastProvider>
  )
}

export default App
