import { invoke } from '@tauri-apps/api/core';

export async function performOcr(imagePath: string, language?: string): Promise<string> {
  return await invoke<string>('perform_ocr', { imagePath, language });
}
