import { Navigation } from './Navigation'
import { MainWindow } from './MainWindow'
import type { LayoutProps } from '../types/layout'

export function Layout({
  navigationItems,
  activeItemId,
  onNavigate,
  onAddAccount,
  hasApiKey,
  riotClientRunning,
  valorantRunning,
  onKillRiotClient,
  onLaunchRiotClient,
  children,
}: LayoutProps) {
  return (
    <div
      data-testid="layout-container"
      className="flex h-screen bg-neutral-800"
    >
      <Navigation
        items={navigationItems}
        activeItemId={activeItemId}
        onNavigate={onNavigate}
        onAddAccount={onAddAccount}
        hasApiKey={hasApiKey}
        riotClientRunning={riotClientRunning}
        valorantRunning={valorantRunning}
        onKillRiotClient={onKillRiotClient}
        onLaunchRiotClient={onLaunchRiotClient}
      />
      <MainWindow>
        {children}
      </MainWindow>
    </div>
  )
}
