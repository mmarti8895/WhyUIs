import { useState } from "react";
import { AppSettings } from "../types";
import { X, Key, Cpu, Thermometer, Lock } from "lucide-react";
import "../styles/modal.css";

interface Props {
  settings: AppSettings;
  onSave: (settings: AppSettings) => void;
  onClearMemory: () => Promise<void>;
  onClose: () => void;
}

export default function SettingsModal({ settings, onSave, onClearMemory, onClose }: Props) {
  const [local, setLocal] = useState<AppSettings>({ ...settings });
  const [isClearingMemory, setIsClearingMemory] = useState(false);
  const [memoryStatus, setMemoryStatus] = useState<string | null>(null);

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setLocal((prev) => ({ ...prev, [key]: value }));
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) onClose();
  };

  const providerLocked = false;

  const handleClearMemory = async () => {
    setIsClearingMemory(true);
    setMemoryStatus(null);
    try {
      await onClearMemory();
      setMemoryStatus("Memory cleared. RoastBot will stop using saved personal context.");
    } catch {
      setMemoryStatus("Failed to clear memory. Please try again.");
    } finally {
      setIsClearingMemory(false);
    }
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
            {providerLocked && (
              <p className="env-notice">
                <Lock size={12} /> Provider locked by <code>.env</code>
              </p>
            )}
            <div className="provider-tabs">
              <button
                className={`provider-tab ${local.provider === "openai" ? "active" : ""}`}
                onClick={() => !providerLocked && update("provider", "openai")}
                disabled={providerLocked}
              >
                OpenAI
              </button>
              <button
                className={`provider-tab ${local.provider === "anthropic" ? "active" : ""}`}
                onClick={() => !providerLocked && update("provider", "anthropic")}
                disabled={providerLocked}
              >
                Anthropic
              </button>
            </div>
          </section>

          {local.provider === "openai" && (
            <section className="settings-section">
              <h3 className="settings-section-title">
                <Key size={15} /> OpenAI Configuration
                {local.openaiFromEnv && (
                  <span className="env-badge"><Lock size={11} /> .env default</span>
                )}
              </h3>
              {local.openaiFromEnv && (
                <p className="env-notice">
                  <Lock size={12} /> Using key from <code>.env</code>. Enter a new key below to override it.
                </p>
              )}
              <div className="form-group">
                <label className="form-label">API Key</label>
                <input
                  type="password"
                  className="form-input"
                  value={local.openaiKey}
                  onChange={(e) => update("openaiKey", e.target.value)}
                  placeholder={local.openaiFromEnv ? "Enter key to override .env…" : "sk-…"}
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
                  <option value="gpt-5.4">gpt-5.4 (latest)</option>
                  <option value="gpt-4.1">gpt-4.1 (latest)</option>
                  <option value="gpt-4.1-mini">gpt-4.1-mini</option>
                  <option value="o4-mini">o4-mini</option>
                  <option value="o3">o3</option>
                  <option value="gpt-4o">gpt-4o</option>
                  <option value="gpt-4o-mini">gpt-4o-mini</option>
                </select>
              </div>
            </section>
          )}

          {local.provider === "anthropic" && (
            <section className="settings-section">
              <h3 className="settings-section-title">
                <Key size={15} /> Anthropic Configuration
                {local.anthropicFromEnv && (
                  <span className="env-badge"><Lock size={11} /> .env default</span>
                )}
              </h3>
              {local.anthropicFromEnv && (
                <p className="env-notice">
                  <Lock size={12} /> Using key from <code>.env</code>. Enter a new key below to override it.
                </p>
              )}
              <div className="form-group">
                <label className="form-label">API Key</label>
                <input
                  type="password"
                  className="form-input"
                  value={local.anthropicKey}
                  onChange={(e) => update("anthropicKey", e.target.value)}
                  placeholder={local.anthropicFromEnv ? "Enter key to override .env…" : "sk-ant-…"}
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
                  <option value="claude-4-sonnet-20241022">claude-4-sonnet (latest)</option>
                  <option value="claude-3-7-sonnet-20250219">claude-3-7-sonnet (latest)</option>
                  <option value="claude-3-5-sonnet-20241022">claude-3-5-sonnet</option>
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

          <section className="settings-section">
            <h3 className="settings-section-title">Memory & Privacy</h3>
            <p className="memory-help-text">
              Clear saved long-term memory (name, preferences, recurring topics) from this device.
            </p>
            <button
              className="btn-danger"
              onClick={handleClearMemory}
              disabled={isClearingMemory}
            >
              {isClearingMemory ? "Clearing..." : "Clear Stored Memory"}
            </button>
            {memoryStatus && <p className="memory-status">{memoryStatus}</p>}
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
