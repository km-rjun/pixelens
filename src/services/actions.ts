import { invoke } from '@tauri-apps/api/core';

export async function executeAction(
  action: string,
  text: string,
  imagePath?: string
): Promise<string> {
  return await invoke<string>('execute_action', { action, text, imagePath });
}

export async function listActions(): Promise<string[]> {
  return await invoke<string[]>('list_actions');
}
