import { useState } from "react";
import { AppSettings } from "../types";
import { X, Key, Cpu, Thermometer } from "lucide-react";
import "../styles/modal.css";

interface Props {
  settings: AppSettings;
  onSave: (settings: AppSettings) => void;
  onClose: () => void;
}

export default function SettingsModal({ settings, onSave, onClose }: Props) {
  const [local, setLocal] = useState<AppSettings>({ ...settings });

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setLocal((prev) => ({ ...prev, [key]: value }));
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) onClose();
  };

  return (
    <div className="modal-backdrop" onClick={handleBackdropClick}>
      <div className="modal-panel">
        <div className="modal-header">
          <h2 className="modal-title">⚙️ Settings</h2>
          <button className="modal-close" onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        <div className="modal-body">
          <section className="settings-section">
            <h3 className="settings-section-title">
              <Cpu size={15} /> LLM Provider
            </h3>
            <div className="provider-tabs">
              <button
                className={`provider-tab ${local.provider === "openai" ? "active" : ""}`}
                onClick={() => update("provider", "openai")}
              >
                OpenAI
              </button>
              <button
                className={`provider-tab ${local.provider === "anthropic" ? "active" : ""}`}
                onClick={() => update("provider", "anthropic")}
              >
                Anthropic
              </button>
            </div>
          </section>

          {local.provider === "openai" && (
            <section className="settings-section">
              <h3 className="settings-section-title">
                <Key size={15} /> OpenAI Configuration
              </h3>
              <div className="form-group">
                <label className="form-label">API Key</label>
                <input
                  type="password"
                  className="form-input"
                  value={local.openaiKey}
                  onChange={(e) => update("openaiKey", e.target.value)}
                  placeholder="sk-..."
                  autoComplete="off"
                />
              </div>
              <div className="form-group">
                <label className="form-label">Model</label>
                <select
                  className="form-select"
                  value={local.openaiModel}
                  onChange={(e) => update("openaiModel", e.target.value)}
                >
                  <option value="gpt-4o">gpt-4o</option>
                  <option value="gpt-4o-mini">gpt-4o-mini</option>
                  <option value="gpt-4-turbo">gpt-4-turbo</option>
                  <option value="gpt-3.5-turbo">gpt-3.5-turbo</option>
                </select>
              </div>
            </section>
          )}

          {local.provider === "anthropic" && (
            <section className="settings-section">
              <h3 className="settings-section-title">
                <Key size={15} /> Anthropic Configuration
              </h3>
              <div className="form-group">
                <label className="form-label">API Key</label>
                <input
                  type="password"
                  className="form-input"
                  value={local.anthropicKey}
                  onChange={(e) => update("anthropicKey", e.target.value)}
                  placeholder="sk-ant-..."
                  autoComplete="off"
                />
              </div>
              <div className="form-group">
                <label className="form-label">Model</label>
                <select
                  className="form-select"
                  value={local.anthropicModel}
                  onChange={(e) => update("anthropicModel", e.target.value)}
                >
                  <option value="claude-3-5-sonnet-20241022">claude-3-5-sonnet (recommended)</option>
                  <option value="claude-3-5-haiku-20241022">claude-3-5-haiku (fast)</option>
                  <option value="claude-3-opus-20240229">claude-3-opus (powerful)</option>
                </select>
              </div>
            </section>
          )}

          <section className="settings-section">
            <h3 className="settings-section-title">
              <Thermometer size={15} /> Roast Intensity
            </h3>
            <div className="form-group">
              <label className="form-label">
                Temperature: <strong>{local.temperature.toFixed(1)}</strong>
              </label>
              <input
                type="range"
                className="form-range"
                min="0.5"
                max="1.5"
                step="0.1"
                value={local.temperature}
                onChange={(e) => update("temperature", parseFloat(e.target.value))}
              />
              <div className="range-labels">
                <span>Mild 🌶️</span>
                <span>Nuclear ☢️</span>
              </div>
            </div>
          </section>
        </div>

        <div className="modal-footer">
          <button className="btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button className="btn-primary" onClick={() => onSave(local)}>
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
}
