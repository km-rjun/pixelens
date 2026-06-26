import { useState, useEffect } from 'react'
import CaptureOverlay from './components/CaptureOverlay'
import ResultPanel from './components/ResultPanel'
import SettingsModal from './components/SettingsModal'
import { checkCaptureTools } from './services/capture'
import { checkOcrTools } from './services/ocr'
import './App.css'

function App() {
  const [isCapturing, setIsCapturing] = useState(false)
  const [capturedImage, setCapturedImage] = useState<string | null>(null)
  const [ocrText, setOcrText] = useState<string>('')
  const [showSettings, setShowSettings] = useState(false)
  const [toolStatus, setToolStatus] = useState<{ capture: string[]; ocr: string[] }>({ capture: [], ocr: [] })
  const [showToolWarning, setShowToolWarning] = useState(false)

  useEffect(() => {
    checkTools()
  }, [])

  const checkTools = async () => {
    try {
      const [captureMissing, ocrMissing] = await Promise.all([
        checkCaptureTools(),
        checkOcrTools()
      ])
      setToolStatus({ capture: captureMissing, ocr: ocrMissing })
      if (captureMissing.length > 0 || ocrMissing.length > 0) {
        setShowToolWarning(true)
      }
    } catch (error) {
      console.error('Failed to check tools:', error)
    }
  }

  const handleCapture = async () => {
    setIsCapturing(true)
  }

  const handleCaptureComplete = (imagePath: string) => {
    setCapturedImage(imagePath)
    setIsCapturing(false)
  }

  const handleCaptureCancel = () => {
    setIsCapturing(false)
  }

  return (
    <div className="app">
      <header className="app-header">
        <h1>Pixelens</h1>
        <button onClick={() => setShowSettings(true)}>Settings</button>
      </header>
      
      {showToolWarning && (
        <div className="tool-warning">
          <p>Missing tools:</p>
          {toolStatus.capture.length > 0 && (
            <p>Capture: {toolStatus.capture.join(', ')}</p>
          )}
          {toolStatus.ocr.length > 0 && (
            <p>OCR: {toolStatus.ocr.join(', ')}</p>
          )}
          <button onClick={() => setShowToolWarning(false)}>Dismiss</button>
        </div>
      )}
      
      <main className="app-main">
        {!capturedImage ? (
          <div className="capture-section">
            <p>Select a region of your screen to capture</p>
            <button onClick={handleCapture}>Capture Region</button>
          </div>
        ) : (
          <ResultPanel
            imagePath={capturedImage}
            ocrText={ocrText}
            onOcrTextChange={setOcrText}
            onNewCapture={() => {
              setCapturedImage(null)
              setOcrText('')
            }}
          />
        )}
      </main>

      {isCapturing && (
        <CaptureOverlay
          onComplete={handleCaptureComplete}
          onCancel={handleCaptureCancel}
        />
      )}

      {showSettings && (
        <SettingsModal onClose={() => setShowSettings(false)} />
      )}
    </div>
  )
}

export default App
