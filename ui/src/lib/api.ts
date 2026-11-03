import { invoke } from "@tauri-apps/api/core";

export interface Limits {
  target: number;
  errorHigh: number;
  warningHigh: number;
  warningLow: number;
  errorLow: number;
}

export type ScannerModel = "NP800" | "NP1200" | "RNP1200";
export const scannerModels: ScannerModel[] = ["NP800", "NP1200", "RNP1200"];

export interface Machine {
  id: number;
  name: string;
  scannerModel: ScannerModel;
  widthMm: number;
  binCount: number;
  speedMMin: number;
  scanTimeS: number;
}

export type MachineInput = Omit<Machine, "id" | "binCount"> & {
  /** null = 800 bins. */
  binCount: number | null;
};

export interface Property {
  id: number;
  machineId: number;
  code: string;
  name: string;
  unit: string;
  decimals: number;
  sortOrder: number;
  active: boolean;
  limits: Limits;
}

export type PropertyInput = Omit<Property, "id" | "sortOrder"> & {
  /** null = ao final. */
  sortOrder: number | null;
};

export interface GeneratorParams {
  cdAmplitude: number;
  mdAmplitude: number;
  noise: number;
  edgeAmplitude: number;
  streakCount: number;
  sheetBreakProbability: number;
  sensorFaultProbability: number;
}

/** Espelha `GeneratorParams::default()` do domínio. */
export const defaultGeneratorParams: GeneratorParams = {
  cdAmplitude: 0.5,
  mdAmplitude: 0.4,
  noise: 0.35,
  edgeAmplitude: 0.8,
  streakCount: 3,
  sheetBreakProbability: 0.002,
  sensorFaultProbability: 0.005,
};

export interface GenerateRequest {
  machineId: number;
  reelId: string;
  startedAt: string;
  scanCount: number | null;
  durationMin: number | null;
  /** null = todas as propriedades ativas. */
  propertyIds: number[] | null;
  params: GeneratorParams;
  seed: number | null;
  overwrite: boolean;
}

export interface ReelFile {
  propertyCode: string;
  propertyName: string;
  filePath: string;
}

export interface ReelRecord {
  id: number;
  reelId: string;
  machineId: number | null;
  machineName: string;
  startedAt: string;
  endedAt: string;
  scanCount: number;
  binCount: number;
  directory: string;
  files: ReelFile[];
  createdAt: string | null;
}

export type ScanDirection = "forward" | "reverse";
export type SensorStatus = "ok" | "notCalibrated" | "dirty" | "fault";

export interface ScanInfo {
  number: number;
  direction: ScanDirection;
  startedAt: string;
  endedAt: string;
  speed: number;
  mdPositionM: number;
  edgeFrontMm: number;
  edgeBackMm: number;
  sheetBreak: boolean;
  sensorStatus: SensorStatus;
  sensorTempC: number;
}

export interface Summary {
  mean: number;
  stdDev: number;
  count: number;
  min: number;
  max: number;
}

export interface ContourData {
  reelId: string;
  filePath: string;
  machine: Omit<Machine, "id"> & { id: number | null };
  property: Omit<Property, "id"> & { id: number | null };
  binCenters: number[];
  scans: ScanInfo[];
  /** Linhas = scans, colunas = bins; null = sem dado. */
  values: (number | null)[][];
  cd: { avg: (number | null)[]; min: (number | null)[]; max: (number | null)[] };
  md: (number | null)[];
  summary: Summary | null;
}

/** Cores do Contour Map por limite (`#rrggbb`). */
export interface ContourColors {
  errorHigh: string;
  warningHigh: string;
  target: string;
  warningLow: string;
  errorLow: string;
}

/** Espelha `ContourColors::default()` do domínio (escala de referência `assets/cores.png`). */
export const defaultContourColors: ContourColors = {
  errorHigh: "#ff8e00",
  warningHigh: "#fae22e",
  target: "#dbff8f",
  warningLow: "#b3d7e5",
  errorLow: "#8381ff",
};

export interface Settings {
  exportDir: string;
  contourColors: ContourColors;
}

export interface AppInfo {
  version: string;
  dbPath: string;
  logDir: string;
}

/** Erro devolvido pelo backend (`application::AppError`). */
export interface AppError {
  code: string;
  field: string | null;
  message: string;
}

export function isAppError(e: unknown): e is AppError {
  return typeof e === "object" && e !== null && "code" in e && "message" in e;
}

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  listMachines: () => invoke<Machine[]>("list_machines"),
  createMachine: (input: MachineInput) => invoke<Machine>("create_machine", { input }),
  updateMachine: (id: number, input: MachineInput) => invoke<Machine>("update_machine", { id, input }),
  deleteMachine: (id: number) => invoke<void>("delete_machine", { id }),
  listProperties: (machineId: number) => invoke<Property[]>("list_properties", { machineId }),
  createProperty: (input: PropertyInput) => invoke<Property>("create_property", { input }),
  updateProperty: (id: number, input: PropertyInput) => invoke<Property>("update_property", { id, input }),
  deleteProperty: (id: number) => invoke<void>("delete_property", { id }),
  generateReel: (request: GenerateRequest) => invoke<ReelRecord>("generate_reel", { request }),
  listReels: (machineId?: number | null, limit?: number) => invoke<ReelRecord[]>("list_reels", { machineId, limit }),
  deleteReel: (id: number, deleteFiles: boolean) => invoke<void>("delete_reel", { id, deleteFiles }),
  loadContour: (reelId: number, propertyCode: string) => invoke<ContourData>("load_contour", { reelId, propertyCode }),
  loadContourFile: (path: string) => invoke<ContourData>("load_contour_file", { path }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) => invoke<Settings>("update_settings", { settings }),
};
