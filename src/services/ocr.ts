import { invoke } from '@tauri-apps/api/core';
import type { OcrResult } from './types';

export async function performOcr(imagePath: string, language?: string): Promise<OcrResult> {
  return await invoke<OcrResult>('perform_ocr', { imagePath, language });
}

export async function checkOcrTools(): Promise<string[]> {
  return await invoke<string[]>('check_ocr_tools');
}
