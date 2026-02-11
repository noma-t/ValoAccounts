import { useState, useEffect } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { getAppDir, getSettings, updateSettings, getDefaultRiotClientServicePath, getDefaultRiotClientDataPath } from '../lib/tauri'
import type { Settings } from '../types/settings'

interface SettingsPageProps {
  onSettingsUpdated?: () => void
}

const SETTING_STYLES = {
  container: "h-full flex flex-col gap-3 py-3 overflow-y-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
  settingRow: "flex flex-col gap-1",
  label: "text-sm font-semibold text-neutral-200",
  valueRow: "flex items-center gap-2",
  input: "flex-1 h-9 bg-neutral-800 border border-neutral-700/50 rounded px-3 text-sm text-white placeholder-neutral-500 focus:outline-none focus:border-neutral-600/50 transition-colors duration-200",
  browseButton: "h-9 w-9 flex items-center justify-center bg-neutral-700 hover:bg-neutral-600 active:bg-neutral-800 text-white rounded transition-all duration-200 border border-neutral-600/50 hover:border-neutral-500/50 flex-shrink-0",
  browseIcon: "w-4 h-4",
  saveButton: "h-9 w-9 flex items-center justify-center bg-neutral-700 hover:bg-neutral-600 active:bg-neutral-800 text-white rounded transition-all duration-200 border border-neutral-600/50 hover:border-neutral-500/50 flex-shrink-0 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-neutral-700 disabled:hover:border-neutral-600/50",
  saveIcon: "w-4 h-4",
}

export function SettingsPage({ onSettingsUpdated }: SettingsPageProps) {
  const [settings, setSettings] = useState<Settings | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [appDir, setAppDir] = useState<string>('')
  const [editedPaths, setEditedPaths] = useState({
    riot_client_service_path: '',
    riot_client_data_path: '',
    account_data_path: '',
    henrikdev_api_key: '',
  })

  useEffect(() => {
    void loadSettings()
    void loadAppDir()
  }, [])

  const loadAppDir = async () => {
    try {
      const dir = await getAppDir()
      setAppDir(dir)
    } catch (err) {
      console.error('Failed to get app directory:', err)
    }
  }

  useEffect(() => {
    const loadDefaults = async () => {
      if (settings) {
        let riotServicePath = settings.riot_client_service_path || ''
        let riotDataPath = settings.riot_client_data_path || ''

        if (!riotServicePath) {
          try {
            riotServicePath = await getDefaultRiotClientServicePath()
          } catch (err) {
            console.error('Failed to get default Riot Client Service path:', err)
          }
        }

        if (!riotDataPath) {
          try {
            riotDataPath = await getDefaultRiotClientDataPath()
          } catch (err) {
            console.error('Failed to get default Riot Client Data path:', err)
          }
        }

        setEditedPaths({
          riot_client_service_path: riotServicePath,
          riot_client_data_path: riotDataPath,
          account_data_path: settings.account_data_path || '',
          henrikdev_api_key: settings.henrikdev_api_key || '',
        })
      }
    }

    void loadDefaults()
  }, [settings])

  const loadSettings = async () => {
    try {
      setLoading(true)
      setError(null)
      const loadedSettings = await getSettings()
      setSettings(loadedSettings)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings')
    } finally {
      setLoading(false)
    }
  }

  const handleBrowseFile = async (settingKey: 'riot_client_service_path') => {
    try {
      const currentPath = editedPaths[settingKey]
      let defaultPath: string | undefined = undefined

      if (currentPath) {
        const lastSlash = Math.max(currentPath.lastIndexOf('\\'), currentPath.lastIndexOf('/'))
        if (lastSlash !== -1) {
          defaultPath = currentPath.substring(0, lastSlash)
        }
      } else if (appDir) {
        defaultPath = appDir
      }

      const selected = await open({
        multiple: false,
        directory: false,
        defaultPath,
      })

      if (selected && typeof selected === 'string') {
        setEditedPaths({
          ...editedPaths,
          [settingKey]: selected,
        })
      }
    } catch (err) {
      console.error('Failed to select file:', err)
      setError(err instanceof Error ? err.message : 'Failed to select file')
    }
  }

  const handleBrowseDirectory = async (settingKey: 'riot_client_data_path' | 'account_data_path') => {
    try {
      const currentPath = editedPaths[settingKey]

      const selected = await open({
        multiple: false,
        directory: true,
        defaultPath: currentPath || appDir || undefined,
      })

      if (selected && typeof selected === 'string') {
        setEditedPaths({
          ...editedPaths,
          [settingKey]: selected,
        })
      }
    } catch (err) {
      console.error('Failed to select directory:', err)
      setError(err instanceof Error ? err.message : 'Failed to select directory')
    }
  }

  const handleSave = async (settingKey: 'riot_client_service_path' | 'riot_client_data_path' | 'account_data_path' | 'henrikdev_api_key') => {
    try {
      const updatedSettings = await updateSettings({
        [settingKey]: editedPaths[settingKey],
      })
      setSettings(updatedSettings)
      onSettingsUpdated?.()
    } catch (err) {
      console.error(`[SettingsPage] Error saving ${settingKey}:`, err)
      setError(err instanceof Error ? err.message : 'Failed to save setting')
    }
  }

  const handleInputChange = (settingKey: 'riot_client_service_path' | 'riot_client_data_path' | 'account_data_path' | 'henrikdev_api_key', value: string) => {
    setEditedPaths({
      ...editedPaths,
      [settingKey]: value,
    })
  }

  if (loading) {
    return (
      <div data-testid="settings-page" className={SETTING_STYLES.container}>
        <div className="text-neutral-400 text-sm">Loading settings...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div data-testid="settings-page" className={SETTING_STYLES.container}>
        <div className="text-red-400 text-sm">Error: {error}</div>
      </div>
    )
  }

  if (!settings) {
    return null
  }

  return (
    <div data-testid="settings-page" className={SETTING_STYLES.container}>
      <div className={SETTING_STYLES.settingRow}>
        <label className={SETTING_STYLES.label}>RiotClientService.exe Path</label>
        <div className={SETTING_STYLES.valueRow}>
          <input
            type="text"
            value={editedPaths.riot_client_service_path}
            onChange={(e) => handleInputChange('riot_client_service_path', e.target.value)}
            placeholder="C:\Riot Games\Riot Client\RiotClientServices.exe"
            className={SETTING_STYLES.input}
          />
          <button
            onClick={() => handleBrowseFile('riot_client_service_path')}
            className={SETTING_STYLES.browseButton}
            title="Browse for file"
          >
            <img src="/folder-icon.svg" alt="Browse" className={SETTING_STYLES.browseIcon} />
          </button>
          <button
            onClick={() => handleSave('riot_client_service_path')}
            className={SETTING_STYLES.saveButton}
            title="Save"
            disabled={editedPaths.riot_client_service_path === (settings.riot_client_service_path ?? '')}
          >
            <img src="/checkmark-icon.svg" alt="Save" className={SETTING_STYLES.saveIcon} />
          </button>
        </div>
      </div>

      <div className={SETTING_STYLES.settingRow}>
        <label className={SETTING_STYLES.label}>Riot Client Data Path</label>
        <div className={SETTING_STYLES.valueRow}>
          <input
            type="text"
            value={editedPaths.riot_client_data_path}
            onChange={(e) => handleInputChange('riot_client_data_path', e.target.value)}
            placeholder="%LOCALAPPDATA%/Riot Games/Riot Client/Data"
            className={SETTING_STYLES.input}
          />
          <button
            onClick={() => handleBrowseDirectory('riot_client_data_path')}
            className={SETTING_STYLES.browseButton}
            title="Browse for directory"
          >
            <img src="/folder-icon.svg" alt="Browse" className={SETTING_STYLES.browseIcon} />
          </button>
          <button
            onClick={() => handleSave('riot_client_data_path')}
            className={SETTING_STYLES.saveButton}
            title="Save"
            disabled={editedPaths.riot_client_data_path === (settings.riot_client_data_path ?? '')}
          >
            <img src="/checkmark-icon.svg" alt="Save" className={SETTING_STYLES.saveIcon} />
          </button>
        </div>
      </div>

      <div className={SETTING_STYLES.settingRow}>
        <label className={SETTING_STYLES.label}>Account Data Path</label>
        <div className={SETTING_STYLES.valueRow}>
          <input
            type="text"
            value={editedPaths.account_data_path}
            onChange={(e) => handleInputChange('account_data_path', e.target.value)}
            placeholder="Account data directory path..."
            className={SETTING_STYLES.input}
          />
          <button
            onClick={() => handleBrowseDirectory('account_data_path')}
            className={SETTING_STYLES.browseButton}
            title="Browse for directory"
          >
            <img src="/folder-icon.svg" alt="Browse" className={SETTING_STYLES.browseIcon} />
          </button>
          <button
            onClick={() => handleSave('account_data_path')}
            className={SETTING_STYLES.saveButton}
            title="Save"
            disabled={editedPaths.account_data_path === (settings.account_data_path ?? '')}
          >
            <img src="/checkmark-icon.svg" alt="Save" className={SETTING_STYLES.saveIcon} />
          </button>
        </div>
      </div>

      <div className={SETTING_STYLES.settingRow}>
        <label className={SETTING_STYLES.label}>Henrikdev API Key</label>
        <div className={SETTING_STYLES.valueRow}>
          <input
            type="text"
            value={editedPaths.henrikdev_api_key}
            onChange={(e) => handleInputChange('henrikdev_api_key', e.target.value)}
            placeholder="Enter your Henrikdev API key..."
            className={SETTING_STYLES.input}
          />
          <button
            onClick={() => handleSave('henrikdev_api_key')}
            className={SETTING_STYLES.saveButton}
            title="Save"
            disabled={editedPaths.henrikdev_api_key === (settings.henrikdev_api_key ?? '')}
          >
            <img src="/checkmark-icon.svg" alt="Save" className={SETTING_STYLES.saveIcon} />
          </button>
        </div>
      </div>
    </div>
  )
}
