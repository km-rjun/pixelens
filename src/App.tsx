import { useState } from 'react'
import CaptureOverlay from './components/CaptureOverlay'
import ResultPanel from './components/ResultPanel'
import SettingsModal from './components/SettingsModal'
import './App.css'

function App() {
  const [isCapturing, setIsCapturing] = useState(false)
  const [capturedImage, setCapturedImage] = useState<string | null>(null)
  const [ocrText, setOcrText] = useState<string>('')
  const [showSettings, setShowSettings] = useState(false)

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
