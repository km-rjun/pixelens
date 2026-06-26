import { useState, useEffect } from 'react'
import { getConfig, saveConfig } from '../services/config'

interface SettingsModalProps {
  onClose: () => void
}

function SettingsModal({ onClose }: SettingsModalProps) {
  const [apiEndpoint, setApiEndpoint] = useState('https://api.openai.com/v1')
  const [apiKey, setApiKey] = useState('')
  const [model, setModel] = useState('gpt-4o')
  const [ocrLanguage, setOcrLanguage] = useState('eng')
  const [hotkey, setHotkey] = useState('Ctrl+Shift+C')
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    loadConfig()
  }, [])

  const loadConfig = async () => {
    try {
      const config = await getConfig()
      setApiEndpoint(config.api_endpoint)
      setApiKey(config.api_key)
      setModel(config.model)
      setOcrLanguage(config.ocr_language)
      setHotkey(config.hotkey)
    } catch (error) {
      console.error('Failed to load config:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleSave = async () => {
    try {
      await saveConfig({
        api_endpoint: apiEndpoint,
        api_key: apiKey,
        model,
        ocr_language: ocrLanguage,
        hotkey,
      })
      onClose()
    } catch (error) {
      console.error('Failed to save config:', error)
    }
  }

  if (isLoading) {
    return (
      <div className="modal-overlay">
        <div className="modal">
          <p>Loading settings...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h2>Settings</h2>
        
        <div className="setting-group">
          <label>AI Provider Endpoint:</label>
          <input
            type="text"
            value={apiEndpoint}
            onChange={(e) => setApiEndpoint(e.target.value)}
            placeholder="https://api.openai.com/v1"
          />
        </div>

        <div className="setting-group">
          <label>API Key:</label>
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="sk-..."
          />
        </div>

        <div className="setting-group">
          <label>Model:</label>
          <input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder="gpt-4o"
          />
        </div>

        <div className="setting-group">
          <label>OCR Language:</label>
          <select value={ocrLanguage} onChange={(e) => setOcrLanguage(e.target.value)}>
            <option value="eng">English</option>
            <option value="spa">Spanish</option>
            <option value="fra">French</option>
            <option value="deu">German</option>
            <option value="ita">Italian</option>
            <option value="por">Portuguese</option>
            <option value="rus">Russian</option>
            <option value="jpn">Japanese</option>
            <option value="kor">Korean</option>
            <option value="chi_sim">Chinese (Simplified)</option>
          </select>
        </div>

        <div className="setting-group">
          <label>Hotkey:</label>
          <input
            type="text"
            value={hotkey}
            onChange={(e) => setHotkey(e.target.value)}
            placeholder="Ctrl+Shift+C"
          />
        </div>

        <div className="modal-actions">
          <button onClick={onClose}>Cancel</button>
          <button onClick={handleSave}>Save</button>
        </div>
      </div>
    </div>
  )
}

export default SettingsModal
