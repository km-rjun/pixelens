import { invoke } from '@tauri-apps/api/core';
import type { Config } from './types';

export async function getConfig(): Promise<Config> {
  return await invoke<Config>('get_config');
}

export async function saveConfig(config: Config): Promise<void> {
  return await invoke<void>('save_config', { config });
}
