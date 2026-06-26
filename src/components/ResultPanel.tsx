import { useState } from 'react'
import { performOcr } from '../services/ocr'
import { askAi } from '../services/ai'
import { executeAction } from '../services/actions'

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
      const result = await performOcr(imagePath)
      onOcrTextChange(result.text)
    } catch (error) {
      console.error('OCR failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleCopy = async () => {
    if (ocrText) {
      try {
        await executeAction('copy', ocrText)
        await navigator.clipboard.writeText(ocrText)
      } catch (error) {
        console.error('Copy failed:', error)
      }
    }
  }

  const handleSearch = async () => {
    if (ocrText) {
      try {
        const url = await executeAction('search', ocrText)
        window.open(url, '_blank')
      } catch (error) {
        console.error('Search failed:', error)
      }
    }
  }

  const handleReverseImage = async () => {
    try {
      const url = await executeAction('reverse_image', '', imagePath)
      window.open(url, '_blank')
    } catch (error) {
      console.error('Reverse image search failed:', error)
    }
  }

  const handleAi = async () => {
    if (!aiPrompt) return
    setIsLoading(true)
    try {
      const result = await askAi(aiPrompt, imagePath)
      setAiResponse(result.content)
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
      const prompt = await executeAction('translate', ocrText)
      const result = await askAi(prompt, imagePath)
      setAiResponse(result.content)
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
        <button onClick={handleReverseImage}>
          Reverse Image Search
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
