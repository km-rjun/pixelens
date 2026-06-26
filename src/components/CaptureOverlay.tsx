import { useEffect } from 'react'
import { captureRegion } from '../services/capture'

interface CaptureOverlayProps {
  onComplete: (imagePath: string) => void
  onCancel: () => void
}

function CaptureOverlay({ onComplete, onCancel }: CaptureOverlayProps) {
  useEffect(() => {
    const doCapture = async () => {
      try {
        const result = await captureRegion()
        onComplete(result.image_path)
      } catch (error) {
        console.error('Capture failed:', error)
        onCancel()
      }
    }
    
    doCapture()
  }, [onComplete, onCancel])

  return (
    <div className="capture-overlay">
      <div className="capture-instructions">
        <p>Select a region on your screen...</p>
        <button onClick={onCancel}>Cancel</button>
      </div>
    </div>
  )
}

export default CaptureOverlay
