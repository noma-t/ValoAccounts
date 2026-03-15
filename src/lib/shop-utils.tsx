import type {
  SkinWeapon,
  BuddyItem,
  PlayercardItem,
  SprayItem,
  FlexItem,
  TitleItem,
} from './tauri'

export type ItemInfo =
  | { kind: 'skin'; data: SkinWeapon }
  | { kind: 'buddy'; data: BuddyItem }
  | { kind: 'playercard'; data: PlayercardItem }
  | { kind: 'spray'; data: SprayItem }
  | { kind: 'flex'; data: FlexItem }
  | { kind: 'title'; data: TitleItem }

export function formatCountdown(totalSecs: number): string {
  if (totalSecs <= 0) return '00:00:00'
  const d = Math.floor(totalSecs / 86400)
  const h = Math.floor((totalSecs % 86400) / 3600)
  const m = Math.floor((totalSecs % 3600) / 60)
  const s = totalSecs % 60
  const hms = `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return d > 0 ? `${d}d ${hms}` : hms
}

export function tierHex(tierColor: string | null): string | null {
  if (!tierColor) return null
  const hex = tierColor.slice(0, 6)
  if (!/^[0-9a-fA-F]{6}$/.test(hex)) return null
  return hex
}

export function cardGradient(hex: string | null): React.CSSProperties {
  if (!hex) {
    return { background: 'linear-gradient(to bottom, #404040 0%, #1a1a1a 100%)' }
  }
  return {
    background: `linear-gradient(to bottom, #${hex}40 0%, #1a1a1a 70%)`,
    borderBottom: `2px solid #${hex}90`,
  }
}

export function skinImageUrl(skin: SkinWeapon | null, levelUuid: string): string {
  if (skin?.display_icon) return skin.display_icon
  return `https://media.valorant-api.com/weaponskinlevels/${levelUuid}/displayicon.png`
}

export function formatVp(vp: number): string {
  return vp.toLocaleString()
}

export function VpIcon() {
  return <img src="/valo-icon.svg" alt="" width={12} height={12} className="opacity-70 block shrink-0" />
}
