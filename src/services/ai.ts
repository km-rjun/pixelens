import { invoke } from '@tauri-apps/api/core';
import type { AiResponse } from './types';

export async function askAi(
  prompt: string,
  imagePath?: string,
  apiEndpoint?: string,
  apiKey?: string,
  model?: string
): Promise<AiResponse> {
  return await invoke<AiResponse>('ask_ai', {
    prompt,
    imagePath,
    apiEndpoint,
    apiKey,
    model,
  });
}
