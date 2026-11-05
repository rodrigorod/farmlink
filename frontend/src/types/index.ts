export type UserRole = 'admin' | 'player';

export interface User {
  id: string;
  username: string;
  email: string;
  role: UserRole;
  token?: string;
}

export type ServerStatus = 'synced' | 'update_required' | 'error_desync';

export interface Server {
  id: string;
  name: string;
  address: string; // e.g. "fs25.farmlink-game.eu:10823"
  mapName: string; // e.g. "Zielonka" or "Riverbend Springs"
  currentPlayers: number;
  maxPlayers: number;
  modpackVersion: string; // e.g. "v2.4.0"
  modCount: number;
  totalSizeBytes: number;
  status: ServerStatus;
  lastUpdated: string;
  isOwner?: boolean;
}

export type LocalModStatus = 'synced' | 'missing' | 'outdated' | 'extra';

export interface ModItem {
  id: string;
  name: string;
  fileName: string; // e.g. "FS25_JohnDeere_7R.zip"
  category: 'Tractors' | 'Harvesters' | 'Tools' | 'Gameplay' | 'Maps' | 'Buildings' | 'Vehicles';
  requiredVersion: string; // e.g. "1.2.0.0"
  localVersion?: string | null;
  status: LocalModStatus;
  sizeBytes: number;
  hashMd5: string;
  hashSha256: string;
  author: string;
  downloadUrl: string;
  thumbnailUrl?: string;
  isEnabled: boolean; // For admin manifest activation toggle
  order: number;
}

export interface CompanionState {
  isConnected: boolean;
  isConnecting: boolean;
  version: string;
  os: 'windows' | 'linux' | 'unknown';
  modsPath: string;
  lastSyncAt: string | null;
  syncInProgress: boolean;
  syncProgress: number; // 0 - 100
  currentSyncMod: string | null;
  simulationMode: boolean;
}

export interface Toast {
  id: string;
  type: 'success' | 'warning' | 'error' | 'info';
  title: string;
  message: string;
  duration?: number;
}
