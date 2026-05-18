export interface PetManifestSummary {
  sourcePath: string;
  header: PetHeaderSummary;
  spriteSheet: SpriteSheetSummary;
  animationCount: number;
  spawnCount: number;
  soundCount: number;
}

export interface AvailablePet {
  id: string;
  xmlPath: string;
  header: PetHeaderSummary;
  animationCount: number;
  spawnCount: number;
  soundCount: number;
}

export type PetAssetSource = "builtin" | "user";

export interface ManageablePet {
  id: string;
  displayName: string;
  author: string;
  xmlPath: string;
  source: PetAssetSource;
  canDelete: boolean;
  animationCount: number;
  spawnCount: number;
  soundCount: number;
}

export interface PetArchiveSummary {
  author: string;
  title: string;
  petName: string;
  version: string;
  animationCount: number;
  spawnCount: number;
  soundCount: number;
  tilesX: number;
  tilesY: number;
  tileWidth: number;
  tileHeight: number;
}

export interface PetArchiveValidation {
  valid: boolean;
  normalizedId: string;
  displayName: string;
  errors: string[];
  warnings: string[];
  summary: PetArchiveSummary | null;
}

export interface ExportedPetArchive {
  fileName: string;
  archiveBase64: string;
}

export interface PetHeaderSummary {
  author: string;
  title: string;
  petname: string;
  version: string;
  info: string;
  application: string;
}

export interface SpriteSheetSummary {
  tilesX: number;
  tilesY: number;
  tileWidth: number;
  tileHeight: number;
  transparency: string;
  dataUrl: string;
}

export interface PetFrame {
  animationId: number;
  animationName: string;
  frameIndex: number;
  sequenceStep: number;
  totalSteps: number;
  x: number;
  y: number;
  width: number;
  height: number;
  intervalMs: number;
  offsetY: number;
  opacity: number;
  flipped: boolean;
}

export interface PetRuntimeStatus {
  selectedXmlPath: string | null;
  activeInstanceId: string | null;
  isRunning: boolean;
  isPaused: boolean;
  windowOpen: boolean;
  latestFrame: PetFrame | null;
  maxInstances: number;
  windowCollisionEnabled: boolean;
  clickThroughEnabled: boolean;
  instances: PetInstanceStatus[];
}

export interface PetInteractionResult {
  status: PetRuntimeStatus;
  usedAnimation: boolean;
}

export interface PetInstanceStatus {
  petInstanceId: string;
  xmlPath: string;
  isPaused: boolean;
  windowOpen: boolean;
  latestFrame: PetFrame | null;
}

export interface AudioStatus {
  muted: boolean;
  lastError: string | null;
}

export interface RectInfo {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface MonitorInfo {
  id: number;
  name: string;
  bounds: RectInfo;
  workArea: RectInfo;
  isPrimary: boolean;
}

export interface PlatformInfo {
  os: string;
  monitors: MonitorInfo[];
  primaryWorkArea: RectInfo;
  taskbarEdge: string | null;
  floorY: number;
  fullscreenWindowActive: boolean;
  windowCollisionAvailable: boolean;
  clickThroughAvailable: boolean;
}
