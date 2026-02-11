export interface NavigationItem {
  id: string
  label: string
  icon: string
}

export interface LayoutProps {
  navigationItems: NavigationItem[]
  activeItemId?: string
  onNavigate?: (itemId: string) => void
  onAddAccount?: () => void
  hasApiKey?: boolean
  riotClientRunning?: boolean
  valorantRunning?: boolean
  onKillRiotClient?: () => void
  onLaunchRiotClient?: () => void
  children?: React.ReactNode
}
