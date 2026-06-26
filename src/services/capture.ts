import { invoke } from '@tauri-apps/api/core';

export async function captureRegion(): Promise<string> {
  return await invoke<string>('capture_region');
}
