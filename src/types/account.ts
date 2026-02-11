export interface Account {
  id: number
  riot_id: string
  tagline: string
  username: string | null
  has_password: boolean
  rank: string | null
  is_active: boolean
  data_folder: string | null
  created_at: string
  updated_at: string
}

export interface CreateAccount {
  riot_id: string
  tagline: string
  username: string | null
  password: string | null
  rank: string | null
  use_current_data: boolean
}

export interface UpdateAccount {
  id: number
  riot_id: string
  tagline: string
  username: string | null
  password: string | null
  rank: string | null
}

export const VALORANT_RANKS = [
  'Unranked',
  'Iron 1', 'Iron 2', 'Iron 3',
  'Bronze 1', 'Bronze 2', 'Bronze 3',
  'Silver 1', 'Silver 2', 'Silver 3',
  'Gold 1', 'Gold 2', 'Gold 3',
  'Platinum 1', 'Platinum 2', 'Platinum 3',
  'Diamond 1', 'Diamond 2', 'Diamond 3',
  'Ascendant 1', 'Ascendant 2', 'Ascendant 3',
  'Immortal 1', 'Immortal 2', 'Immortal 3',
  'Radiant',
] as const

export type ValorantRank = typeof VALORANT_RANKS[number]

export const RANK_ICON_MAP: Record<ValorantRank, string> = {
  'Unranked': 'unranked',
  'Iron 1': 'ir1', 'Iron 2': 'ir2', 'Iron 3': 'ir3',
  'Bronze 1': 'b1', 'Bronze 2': 'b2', 'Bronze 3': 'b3',
  'Silver 1': 's1', 'Silver 2': 's2', 'Silver 3': 's3',
  'Gold 1': 'g1', 'Gold 2': 'g2', 'Gold 3': 'g3',
  'Platinum 1': 'p1', 'Platinum 2': 'p2', 'Platinum 3': 'p3',
  'Diamond 1': 'd1', 'Diamond 2': 'd2', 'Diamond 3': 'd3',
  'Ascendant 1': 'a1', 'Ascendant 2': 'a2', 'Ascendant 3': 'a3',
  'Immortal 1': 'im1', 'Immortal 2': 'im2', 'Immortal 3': 'im3',
  'Radiant': 'r',
}
