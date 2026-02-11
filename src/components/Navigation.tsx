import { useState } from 'react'
import type { NavigationItem } from '../types/layout'

interface NavigationProps {
  items: NavigationItem[]
  activeItemId?: string
  onNavigate?: (itemId: string) => void
  onAddAccount?: () => void
  riotClientRunning?: boolean
  valorantRunning?: boolean
  onKillRiotClient?: () => void
  onLaunchRiotClient?: () => void
}

export function Navigation({
  items,
  activeItemId,
  onNavigate,
  onAddAccount,
  riotClientRunning = false,
  valorantRunning = false,
  onKillRiotClient,
  onLaunchRiotClient,
}: NavigationProps) {
  const [isRiotClientHovered, setIsRiotClientHovered] = useState(false)
  return (
    <nav
      data-testid="navigation-sidebar"
      className="bg-neutral-950 text-white w-52 pt-2 pb-3 flex flex-col border-r border-neutral-900"
    >
      <div className="flex flex-col">
        {items.map((item) => (
          <button
            key={item.id}
            onClick={() => onNavigate?.(item.id)}
            className={`w-full text-left px-6 py-2 transition-all border-l-4 font-medium ${
              activeItemId === item.id
                ? 'active bg-neutral-900 border-white'
                : 'bg-neutral-950 hover:bg-neutral-900 border-transparent'
            }`}
            type="button"
          >
            {item.label}
          </button>
        ))}
      </div>

      <div className="mt-auto px-3 flex flex-col">
        <button
          className="w-full px-4 mb-2 py-1.5 bg-neutral-900 border border-neutral-600/50 hover:border-neutral-500/50 active:bg-neutral-800 rounded-md text-sm transition-all duration-150"
          type="button"
          onClick={onAddAccount}
        >
          + Add Account
        </button>

        <div>
          <div className="py-1 flex items-center gap-2">
            <div className="flex-1 h-px bg-neutral-800"></div>
            <span className="text-xs font-semibold text-neutral-500 uppercase tracking-wider">PROCESS</span>
            <div className="flex-1 h-px bg-neutral-800"></div>
          </div>
          <div className="flex flex-col gap-2">
            <button
                className="w-full px-4 py-1.5 bg-neutral-900 border border-neutral-600/50 rounded-md text-sm transition-all duration-150 relative flex items-center justify-center cursor-default"
                type="button"
            >
              <span
                className={`absolute left-3 w-2.5 h-2.5 rounded-full ${
                  valorantRunning
                    ? 'bg-green-500 border border-green-800'
                    : 'bg-neutral-500 border border-neutral-700'
                }`}
              ></span>
              <span>Valorant</span>
            </button>
            <button
                className="w-full px-4 py-1.5 bg-neutral-900 border border-neutral-600/50 hover:border-neutral-500/50 active:bg-neutral-800 rounded-md text-sm transition-all duration-150 relative flex items-center justify-center overflow-hidden"
                type="button"
                onMouseEnter={() => setIsRiotClientHovered(true)}
                onMouseLeave={() => setIsRiotClientHovered(false)}
                onClick={() => {
                  if (riotClientRunning) {
                    onKillRiotClient?.()
                  } else {
                    onLaunchRiotClient?.()
                  }
                }}
            >
              <span
                className={`absolute left-3 w-2.5 h-2.5 rounded-full ${
                  riotClientRunning
                    ? 'bg-green-500 border border-green-800'
                    : 'bg-neutral-500 border border-neutral-700'
                }`}
              ></span>
              <span
                className={`transition-all duration-150 ${
                  isRiotClientHovered ? 'blur-xs opacity-50' : 'blur-0 opacity-100'
                }`}
              >
                Riot Client
              </span>
              <span
                className={`absolute inset-0 flex items-center justify-center text-sm font-medium transition-all duration-150 ${
                  isRiotClientHovered ? 'opacity-100' : 'opacity-0'
                } ${riotClientRunning ? 'text-red-400' : 'text-green-400'}`}
              >
                {riotClientRunning ? 'Kill' : 'Launch'}
              </span>
            </button>
          </div>
        </div>

      </div>
    </nav>
  )
}
