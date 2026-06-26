import { invoke } from '@tauri-apps/api/core';

export async function askAi(
  prompt: string,
  imagePath?: string,
  apiEndpoint?: string,
  apiKey?: string,
  model?: string
): Promise<string> {
  return await invoke<string>('ask_ai', {
    prompt,
    imagePath,
    apiEndpoint: apiEndpoint || 'https://api.openai.com/v1',
    apiKey: apiKey || '',
    model,
  });
}
