import { useState, useEffect, useCallback } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  getAccountCookies,
  getShop,
  getSkinInfoBatch,
  getBuddyInfoBatch,
  getPlayercardInfoBatch,
  getSprayInfoBatch,
  getFlexInfoBatch,
  getTitleInfoBatch,
  isDemoMode,
  getRiotClientStatus,
  getValorantStatus,
  ITEM_TYPE_SKIN,
} from '../lib/tauri'
import type {
  Storefront,
  SkinWeapon,
  BuddyItem,
  PlayercardItem,
  SprayItem,
  FlexItem,
  TitleItem,
} from '../lib/tauri'
import { useCountdown } from '../hooks/useCountdown'
import { tierHex } from '../lib/shop-utils'
import type { ItemInfo } from '../lib/shop-utils'
import { MOCK_STOREFRONT, MOCK_SKIN_MAP, MOCK_ITEM_MAP } from '../lib/shop-mock-data'
import {
  SectionHeader,
  SkinCard,
  NightMarketCard,
  BundleGroup,
  AccessoryItemCard,
} from '../components/shop'
import '../App.css'

interface ShopWindowProps {
  accountId: number
}

export function ShopWindow({ accountId }: ShopWindowProps) {
  const [storefront, setStorefront] = useState<Storefront | null>(null)
  const [skinMap, setSkinMap] = useState<Record<string, SkinWeapon | null>>({})
  const [itemMap, setItemMap] = useState<Record<string, ItemInfo | null>>({})
  const [loading, setLoading] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isProcessRunning, setIsProcessRunning] = useState(false)

  const dailyRemaining = useCountdown(storefront?.daily_remaining_secs ?? null)
  const accessoriesRemaining = useCountdown(storefront?.accessories_remaining_secs ?? null)
  const nightmarketRemaining = useCountdown(storefront?.night_market_remaining_secs ?? null)

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') getCurrentWindow().close()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  const fetchShop = useCallback((force = false) => {
    isDemoMode().then((isDemo) => {
      if (isDemo) {
        setStorefront(MOCK_STOREFRONT)
        setSkinMap(MOCK_SKIN_MAP)
        setItemMap(MOCK_ITEM_MAP)
        return
      }

      if (force) {
        setRefreshing(true)
      } else {
        setLoading(true)
        setStorefront(null)
      }
      setError(null)

      getAccountCookies(accountId)
        .then(async (cookies) => {
          if (!cookies) {
            setError('No session found. Please log in with Riot Client first.')
            return
          }

          const sf = await getShop(accountId, cookies, force)
          setStorefront(sf)

          const skinUuids = [
            ...sf.daily_offers.map((o) => o.skin_uuid),
            ...(sf.night_market ?? []).map((o) => o.skin_uuid),
            ...(sf.bundles ?? []).flatMap((b) =>
              b.items.filter((i) => i.item_type_id === ITEM_TYPE_SKIN).map((i) => i.item_uuid)
            ),
          ]

          const bundleItems = (sf.bundles ?? []).flatMap((b) => b.items)
          const bonusBundleUuids = bundleItems
            .filter((i) => i.item_type_id !== ITEM_TYPE_SKIN)
            .map((i) => i.item_uuid)

          const accessoryUuids = (sf.accessories ?? []).map((a) => a.item_uuid)
          const allBonusUuids = [...new Set([...bonusBundleUuids, ...accessoryUuids])]

          const fetches = await Promise.allSettled([
            skinUuids.length > 0 ? getSkinInfoBatch(skinUuids) : Promise.resolve([]),
            allBonusUuids.length > 0 ? getBuddyInfoBatch(allBonusUuids) : Promise.resolve([]),
            allBonusUuids.length > 0 ? getPlayercardInfoBatch(allBonusUuids) : Promise.resolve([]),
            allBonusUuids.length > 0 ? getSprayInfoBatch(allBonusUuids) : Promise.resolve([]),
            allBonusUuids.length > 0 ? getFlexInfoBatch(allBonusUuids) : Promise.resolve([]),
            allBonusUuids.length > 0 ? getTitleInfoBatch(allBonusUuids) : Promise.resolve([]),
          ])

          const newSkinMap: Record<string, SkinWeapon | null> = {}
          const newItemMap: Record<string, ItemInfo | null> = {}

          if (fetches[0].status === 'fulfilled') {
            const results = fetches[0].value as (SkinWeapon | null)[]
            skinUuids.forEach((uuid, i) => { newSkinMap[uuid] = results[i] ?? null })
            skinUuids.forEach((uuid, i) => {
              const d = results[i]
              newItemMap[uuid] = d ? { kind: 'skin', data: d } : null
            })
          }

          const buddyResults = fetches[1].status === 'fulfilled'
            ? fetches[1].value as (BuddyItem | null)[]
            : new Array<BuddyItem | null>(allBonusUuids.length).fill(null)
          const cardResults = fetches[2].status === 'fulfilled'
            ? fetches[2].value as (PlayercardItem | null)[]
            : new Array<PlayercardItem | null>(allBonusUuids.length).fill(null)
          const sprayResults = fetches[3].status === 'fulfilled'
            ? fetches[3].value as (SprayItem | null)[]
            : new Array<SprayItem | null>(allBonusUuids.length).fill(null)
          const flexResults = fetches[4].status === 'fulfilled'
            ? fetches[4].value as (FlexItem | null)[]
            : new Array<FlexItem | null>(allBonusUuids.length).fill(null)
          const titleResults = fetches[5].status === 'fulfilled'
            ? fetches[5].value as (TitleItem | null)[]
            : new Array<TitleItem | null>(allBonusUuids.length).fill(null)

          allBonusUuids.forEach((uuid, i) => {
            const buddy = buddyResults[i]
            const card = cardResults[i]
            const spray = sprayResults[i]
            const flex = flexResults[i]
            const title = titleResults[i]
            if (buddy) newItemMap[uuid] = { kind: 'buddy', data: buddy }
            else if (card) newItemMap[uuid] = { kind: 'playercard', data: card }
            else if (spray) newItemMap[uuid] = { kind: 'spray', data: spray }
            else if (flex) newItemMap[uuid] = { kind: 'flex', data: flex }
            else if (title) newItemMap[uuid] = { kind: 'title', data: title }
            else newItemMap[uuid] = null
          })

          setSkinMap(newSkinMap)
          setItemMap(newItemMap)
        })
        .catch((e) => setError(String(e)))
        .finally(() => {
          setLoading(false)
          setRefreshing(false)
        })
    })
  }, [accountId])

  useEffect(() => {
    fetchShop()
  }, [fetchShop])

  useEffect(() => {
    const checkProcesses = () => {
      Promise.all([getRiotClientStatus(), getValorantStatus()])
        .then(([riotRunning, valoRunning]) => {
          setIsProcessRunning(riotRunning || valoRunning)
        })
        .catch(() => setIsProcessRunning(false))
    }

    checkProcesses()
    const interval = setInterval(checkProcesses, 3000)
    return () => clearInterval(interval)
  }, [])

  const bundles = storefront?.bundles ?? []
  const accessories = storefront?.accessories ?? null
  const nightMarket = storefront?.night_market ?? null

  return (
    <div className="min-h-screen bg-neutral-900 text-white flex flex-col">
      <div className="flex-1 overflow-y-auto shop-scrollbar p-6">
        {loading ? (
          <div className="text-sm text-neutral-400 text-center py-8">
            <img src="/refresh-icon.svg" alt="" className="w-5 h-5 animate-spin inline-block" />
          </div>
        ) : error ? (
          <div className="text-sm text-red-400 text-center py-8">{error}</div>
        ) : storefront ? (
          <div className="flex flex-col gap-8">

            <section>
              <SectionHeader
                label="Daily"
                countdown={dailyRemaining}
                onRefresh={() => fetchShop(true)}
                refreshDisabled={refreshing || loading || isProcessRunning}
                refreshing={refreshing}
              />
              <div className="grid grid-cols-4 gap-4">
                {storefront.daily_offers.map((offer) => {
                  const skin = skinMap[offer.skin_uuid] ?? null
                  const hex = tierHex(skin?.tier_color ?? null)
                  return <SkinCard key={offer.skin_uuid} skin={skin} offer={offer} hex={hex} />
                })}
              </div>
            </section>

            {bundles.length > 0 && (
              <section>
                <SectionHeader label="Bundles" />
                <div className="flex flex-col gap-6">
                  {bundles.map((bundle, i) => (
                    <BundleGroup key={i} bundle={bundle} itemMap={itemMap} />
                  ))}
                </div>
              </section>
            )}

            {accessories && accessories.length > 0 && (
              <section>
                <SectionHeader label="Accessories" countdown={accessoriesRemaining} />
                <div className="grid grid-cols-4 gap-4">
                  {accessories.map((offer) => (
                    <AccessoryItemCard
                      key={offer.item_uuid}
                      offer={offer}
                      info={itemMap[offer.item_uuid] ?? null}
                    />
                  ))}
                </div>
              </section>
            )}

            {nightMarket && nightMarket.length > 0 && (
              <section>
                <SectionHeader label="Night Market" countdown={nightmarketRemaining} />
                <div className="grid grid-cols-6 gap-4">
                  {nightMarket.map((offer) => {
                    const skin = skinMap[offer.skin_uuid] ?? null
                    const hex = tierHex(skin?.tier_color ?? null)
                    return (
                      <NightMarketCard
                        key={offer.skin_uuid}
                        skin={skin}
                        offer={offer}
                        hex={hex}
                      />
                    )
                  })}
                </div>
              </section>
            )}

          </div>
        ) : null}
      </div>
    </div>
  )
}
