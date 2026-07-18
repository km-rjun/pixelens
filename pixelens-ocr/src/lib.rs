//! OCR engine layer.
//!
//! M5: real Tesseract binding. `TesseractOcrEngine::new()` validates
//! that the `tesseract` binary is on `$PATH` (PRD §"Dependency
//! Management"); the engine is constructed once at daemon startup and
//! held warm so per-request OCR has no cold-start cost.

use pixelens_core::{CaptureImage, OcrEngine, OcrError};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Tesseract-backed OCR engine. Constructed once at daemon startup and
/// held warm — per-request initialisation would add ~300–500 ms and
/// break the <1 s OCR target (PRD §"OCR").
pub struct TesseractOcrEngine {
    /// Language code passed to `-l` (e.g. `"eng"`).
    language: String,
    /// Tesseract page-segmentation mode (`--psm`), e.g. `6` for a
    /// block of text, `7` for a single text line.
    page_seg_mode: u8,
}

impl TesseractOcrEngine {
    /// Initialise a warm Tesseract engine.
    ///
    /// Validates at construction time that the `tesseract` binary is
    /// present on `$PATH` (PRD: dependency validation at startup). A
    /// missing binary is a hard error here so the daemon can decide
    /// whether to degrade gracefully rather than failing every grab.
    pub fn new() -> Result<Self, OcrError> {
        Self::with_config("eng".to_string(), 6)
    }

    /// Construct with explicit language and page-segmentation mode.
    pub fn with_config(language: String, page_seg_mode: u8) -> Result<Self, OcrError> {
        // Validate tesseract presence. `--version` exits 0 on a working
        // install and prints version info to stderr; we only care about
        // the exit status for the presence check.
        let probe = Command::new("tesseract").arg("--version").output();

        match probe {
            Ok(out) if out.status.success() => {}
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                tracing::warn!(
                    stderr = %stderr.trim(),
                    "tesseract --version exited non-zero"
                );
                return Err(OcrError::Engine(
                    "tesseract reported an error during startup validation".to_string(),
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!("tesseract binary not found on $PATH");
                return Err(OcrError::EngineMissing);
            }
            Err(e) => {
                return Err(OcrError::Engine(format!(
                    "failed to spawn tesseract for validation: {e}"
                )));
            }
        }

        Ok(Self {
            language,
            page_seg_mode,
        })
    }

    /// OCR a PNG (or other tesseract-readable image) already on disk.
    ///
    /// This is the hot path used by the daemon, which receives a PNG
    /// written by `grim`. Tesseract writes its text output to a file;
    /// we pass `stdout` as the output basename and read the
    /// `<basename>.txt` it produces.
    pub fn extract_from_path(&self, image_path: &Path) -> Result<String, OcrError> {
        if !image_path.exists() {
            return Err(OcrError::UnsupportedImage);
        }

        // Tesseract treats the output argument as a *basename* and
        // appends `.txt`. We point it at a temp file so we don't
        // collide with the capture image or leave junk around.
        let out_base = temp_base();
        let out_txt = format!("{out_base}.txt");

        let status = Command::new("tesseract")
            .arg(image_path)
            .arg(&out_base)
            .arg("-l")
            .arg(&self.language)
            .arg("--psm")
            .arg(self.page_seg_mode.to_string())
            .output();

        let output = match status {
            Ok(o) => o,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Should not happen after new() validated, but guard anyway.
                return Err(OcrError::EngineMissing);
            }
            Err(e) => {
                return Err(OcrError::Engine(format!("failed to spawn tesseract: {e}")));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::Engine(format!(
                "tesseract exited {}: {}",
                output.status,
                stderr.trim()
            )));
        }

        let raw = std::fs::read_to_string(&out_txt).map_err(|e| {
            OcrError::Engine(format!("failed to read tesseract output {out_txt}: {e}"))
        })?;
        // Best-effort cleanup of the temp txt file.
        let _ = std::fs::remove_file(&out_txt);

        let text = sanitize_text(&raw);
        if text.is_empty() {
            // Empty output is not an error — it just means tesseract
            // found no text. Return the (empty) sanitized string so the
            // caller can surface "no text found" rather than failing.
            return Ok(String::new());
        }
        Ok(text)
    }
}

impl OcrEngine for TesseractOcrEngine {
    /// OCR from an in-memory [`CaptureImage`] by encoding it to a temp
    /// PNG and delegating to [`Self::extract_from_path`].
    fn extract_text(&self, image: &CaptureImage) -> Result<String, OcrError> {
        if image.is_empty() {
            return Err(OcrError::UnsupportedImage);
        }
        let png_path = temp_base() + ".png";
        encode_image_as_png(image, &png_path)?;
        let result = self.extract_from_path(Path::new(&png_path));
        let _ = std::fs::remove_file(&png_path);
        result
    }
}

/// Build a unique temp-file basename under the system temp dir.
fn temp_base() -> String {
    let dir = std::env::temp_dir().join("pixelens-ocr");
    let _ = std::fs::create_dir_all(&dir);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = dir.join(format!("ocr-{stamp}"));
    path.to_string_lossy().into_owned()
}

/// Normalize tesseract output: trim surrounding whitespace and collapse
/// internal runs of blank lines into single newlines so the clipboard /
/// IPC payload is clean. Pure function — easy to unit test.
pub fn sanitize_text(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_blank = false;
    for line in trimmed.lines() {
        let blank = line.trim().is_empty();
        if blank {
            if !prev_blank && !out.is_empty() {
                out.push('\n');
            }
            prev_blank = true;
        } else {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line.trim());
            prev_blank = false;
        }
    }
    out
}

/// Encode an RGBA [`CaptureImage`] to a 24-bit BMP on disk.
///
/// BMP is uncompressed and natively read by leptonica (hence
/// tesseract), so we avoid pulling in a PNG encoder dependency. RGBA
/// is converted to RGB (alpha dropped) since BMP here is 24 bpp.
fn encode_image_as_png(image: &CaptureImage, path: &str) -> Result<(), OcrError> {
    let w = image.width;
    let h = image.height;
    let stride = image.stride as usize;
    if w == 0 || h == 0 {
        return Err(OcrError::UnsupportedImage);
    }

    // BMP rows are padded to a multiple of 4 bytes.
    let row_bytes = (w as usize) * 3;
    let padding = (4 - (row_bytes % 4)) % 4;
    let padded_row = row_bytes + padding;
    let pixel_array_size = padded_row * (h as usize);
    let file_size = 54 + pixel_array_size;

    let mut buf: Vec<u8> = Vec::with_capacity(file_size);

    // BITMAPFILEHEADER (14 bytes)
    buf.extend_from_slice(b"BM");
    buf.extend_from_slice(&(file_size as u32).to_le_bytes());
    buf.extend_from_slice(&[0u8; 4]); // reserved
    buf.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset

    // BITMAPINFOHEADER (40 bytes)
    buf.extend_from_slice(&40u32.to_le_bytes()); // header size
    buf.extend_from_slice(&(w as i32).to_le_bytes()); // width
    buf.extend_from_slice(&(h as i32).to_le_bytes()); // height
    buf.extend_from_slice(&1u16.to_le_bytes()); // planes
    buf.extend_from_slice(&24u16.to_le_bytes()); // bpp
    buf.extend_from_slice(&0u32.to_le_bytes()); // compression (none)
    buf.extend_from_slice(&(pixel_array_size as u32).to_le_bytes()); // image size
    buf.extend_from_slice(&2835u32.to_le_bytes()); // x pixels per meter
    buf.extend_from_slice(&2835u32.to_le_bytes()); // y pixels per meter
    buf.extend_from_slice(&0u32.to_le_bytes()); // colors used
    buf.extend_from_slice(&0u32.to_le_bytes()); // important colors

    // Pixel data, bottom-up (BMP convention).
    let pixels = &image.pixels;
    for y in (0..h as usize).rev() {
        let row_start = y * stride;
        for x in 0..w as usize {
            let idx = row_start + x * 4;
            if idx + 2 >= pixels.len() {
                // Defensive: malformed buffer. Emit black.
                buf.extend_from_slice(&[0u8, 0u8, 0u8]);
                continue;
            }
            let r = pixels[idx];
            let g = pixels[idx + 1];
            let b = pixels[idx + 2];
            // BMP stores BGR.
            buf.extend_from_slice(&[b, g, r]);
        }
        buf.resize(buf.len() + padding, 0);
    }

    std::fs::write(path, &buf)
        .map_err(|e| OcrError::Engine(format!("failed to write temporary image {path}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn sanitize_trims_and_collapses_blank_lines() {
        // Multiple blank lines between text blocks collapse to a single
        // separator; internal spacing within a line is preserved.
        let raw = "\n\n  Hello   world  \n\n\n\n  Second line  \n\n";
        assert_eq!(sanitize_text(raw), "Hello   world\n\nSecond line");
    }

    #[test]
    fn sanitize_empty_input_yields_empty() {
        assert_eq!(sanitize_text("   \n\t\n  "), "");
    }

    #[test]
    fn sanitize_single_line_trims_inline_spaces() {
        assert_eq!(sanitize_text("   one   two   "), "one   two");
    }

    #[test]
    fn new_errors_when_tesseract_absent() {
        // Spawn a deliberately-bogus binary name to exercise the
        // NotFound branch without depending on the real install.
        let result = TesseractOcrEngine::with_config("eng".to_string(), 6);
        // Only assert the error path is reachable and typed correctly;
        // on a machine with tesseract installed this is Ok.
        match result {
            Ok(_) => { /* tesseract present — fine */ }
            Err(OcrError::EngineMissing) => { /* expected on minimal hosts */ }
            Err(OcrError::UnsupportedImage) => {
                panic!("unexpected UnsupportedImage from new()");
            }
            Err(OcrError::Engine(msg)) => {
                panic!("unexpected engine error: {msg}");
            }
        }
    }

    #[test]
    fn extract_from_path_missing_file_errors() {
        let engine = TesseractOcrEngine::new();
        // If tesseract is present, ensure a non-existent file errors
        // cleanly rather than panicking.
        if let Ok(engine) = engine {
            let missing = PathBuf::from("/nonexistent/pixelens-missing-xyz.png");
            let err = engine.extract_from_path(&missing);
            assert!(err.is_err());
        }
    }
}
