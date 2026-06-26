import { useState } from 'react'
import { performOcr } from '../services/ocr'
import { askAi } from '../services/ai'

interface ResultPanelProps {
  imagePath: string
  ocrText: string
  onOcrTextChange: (text: string) => void
  onNewCapture: () => void
}

function ResultPanel({ imagePath, ocrText, onOcrTextChange, onNewCapture }: ResultPanelProps) {
  const [isLoading, setIsLoading] = useState(false)
  const [aiResponse, setAiResponse] = useState('')
  const [aiPrompt, setAiPrompt] = useState('')

  const handleOcr = async () => {
    setIsLoading(true)
    try {
      const text = await performOcr(imagePath)
      onOcrTextChange(text)
    } catch (error) {
      console.error('OCR failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleCopy = async () => {
    if (ocrText) {
      await navigator.clipboard.writeText(ocrText)
    }
  }

  const handleSearch = () => {
    if (ocrText) {
      window.open(`https://www.google.com/search?q=${encodeURIComponent(ocrText)}`, '_blank')
    }
  }

  const handleAi = async () => {
    if (!aiPrompt) return
    setIsLoading(true)
    try {
      const response = await askAi(aiPrompt, imagePath)
      setAiResponse(response)
    } catch (error) {
      console.error('AI request failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleTranslate = async () => {
    if (!ocrText) return
    setIsLoading(true)
    try {
      const response = await askAi(`Translate this text to English: ${ocrText}`, imagePath)
      setAiResponse(response)
    } catch (error) {
      console.error('Translation failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="result-panel">
      <div className="image-preview">
        <img src={`asset://localhost/${imagePath}`} alt="Captured" />
      </div>
      
      <div className="actions">
        <button onClick={handleOcr} disabled={isLoading}>
          {isLoading ? 'Processing...' : 'Extract Text (OCR)'}
        </button>
        <button onClick={handleCopy} disabled={!ocrText}>
          Copy Text
        </button>
        <button onClick={handleSearch} disabled={!ocrText}>
          Search Text
        </button>
        <button onClick={handleTranslate} disabled={!ocrText || isLoading}>
          Translate
        </button>
        <button onClick={onNewCapture}>
          New Capture
        </button>
      </div>

      {ocrText && (
        <div className="ocr-result">
          <h3>Extracted Text:</h3>
          <textarea value={ocrText} readOnly rows={6} />
        </div>
      )}

      <div className="ai-section">
        <h3>Ask AI:</h3>
        <textarea
          value={aiPrompt}
          onChange={(e) => setAiPrompt(e.target.value)}
          placeholder="Ask a question about the image..."
          rows={3}
        />
        <button onClick={handleAi} disabled={!aiPrompt || isLoading}>
          Ask AI
        </button>
      </div>

      {aiResponse && (
        <div className="ai-response">
          <h3>AI Response:</h3>
          <textarea value={aiResponse} readOnly rows={6} />
        </div>
      )}
    </div>
  )
}

export default ResultPanel
