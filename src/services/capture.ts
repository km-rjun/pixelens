import { invoke } from '@tauri-apps/api/core';
import type { CaptureResult } from './types';

export async function captureRegion(): Promise<CaptureResult> {
  return await invoke<CaptureResult>('capture_region');
}

export async function checkCaptureTools(): Promise<string[]> {
  return await invoke<string[]>('check_capture_tools');
}
