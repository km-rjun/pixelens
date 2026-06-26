export interface CaptureRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CaptureResult {
  image_path: string;
  region: CaptureRegion;
}

export interface OcrResult {
  text: string;
  language: string;
}

export interface AiResponse {
  content: string;
  model: string;
}

export interface Config {
  api_endpoint: string;
  api_key: string;
  model: string;
  ocr_language: string;
  hotkey: string;
}
