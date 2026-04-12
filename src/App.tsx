import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./components/Sidebar";
import ChatWindow from "./components/ChatWindow";
import SettingsModal from "./components/SettingsModal";
import { Message, ChatSession, AppSettings } from "./types";
import "./styles/app.css";

const createSession = (): ChatSession => ({
  id: crypto.randomUUID(),
  title: "New Roast Session",
  messages: [],
  createdAt: Date.now(),
});

const defaultSettings: AppSettings = {
  provider: "openai",
  openaiKey: "",
  openaiModel: "gpt-4.1",
  anthropicKey: "",
  anthropicModel: "claude-3-7-sonnet-20250219",
  temperature: 0.9,
  openaiFromEnv: false,
  anthropicFromEnv: false,
};

export default function App() {
  const [sessions, setSessions] = useState<ChatSession[]>([createSession()]);
  const [activeSessionId, setActiveSessionId] = useState<string>(sessions[0].id);
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [showSettings, setShowSettings] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [showMissingKeyBanner, setShowMissingKeyBanner] = useState(false);
  const [showInvalidKeyBanner, setShowInvalidKeyBanner] = useState(false);

  const hasConfiguredKey = (s: AppSettings): boolean => {
    return Boolean(s.openaiKey) || Boolean(s.anthropicKey);
  };

  // Load settings from the backend on mount (keys are masked, env flags are set).
  useEffect(() => {
    invoke<AppSettings>("load_settings")
      .then((s) => {
        setSettings(s);
        setShowMissingKeyBanner(!hasConfiguredKey(s));
      })
      .catch(console.error);
  }, []);

  const activeSession = sessions.find((s) => s.id === activeSessionId)!;

  const sendMessage = useCallback(
    async (content: string) => {
      if (!content.trim() || isLoading) return;

      const userMessage: Message = {
        id: crypto.randomUUID(),
        role: "user",
        content,
        timestamp: Date.now(),
      };

      setSessions((prev) =>
        prev.map((s) =>
          s.id === activeSessionId
            ? {
                ...s,
                title: s.messages.length === 0 ? content.slice(0, 40) : s.title,
                messages: [...s.messages, userMessage],
              }
            : s
        )
      );

      setIsLoading(true);

      try {
        const response = await invoke<string>("send_message", {
          message: content,
          sessionId: activeSessionId,
          history: activeSession.messages.slice(-14).map((m) => ({
            role: m.role,
            content: m.content,
          })),
        });

        setShowInvalidKeyBanner(false);

        const assistantMessage: Message = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: response,
          timestamp: Date.now(),
        };

        setSessions((prev) =>
          prev.map((s) =>
            s.id === activeSessionId
              ? { ...s, messages: [...s.messages, assistantMessage] }
              : s
          )
        );
      } catch (err) {
        const errText = String(err);
        const missingKey = errText.includes("API key is not configured");
        const invalidKey =
          errText.includes("OpenAI error") ||
          errText.includes("Anthropic error") ||
          errText.toLowerCase().includes("invalid api key") ||
          errText.includes("HTTP 401");

        if (missingKey) {
          setShowMissingKeyBanner(true);
          setShowInvalidKeyBanner(false);
        } else if (invalidKey) {
          setShowInvalidKeyBanner(true);
        }

        const errorMessage: Message = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: `⚠️ **Error:** ${errText}\n\nConfigure your API keys in Settings (⚙️) to get roasted properly.`,
          timestamp: Date.now(),
        };
        setSessions((prev) =>
          prev.map((s) =>
            s.id === activeSessionId
              ? { ...s, messages: [...s.messages, errorMessage] }
              : s
          )
        );
      } finally {
        setIsLoading(false);
      }
    },
    [activeSessionId, activeSession.messages, isLoading, settings]
  );

  const newSession = () => {
    const session = createSession();
    setSessions((prev) => [session, ...prev]);
    setActiveSessionId(session.id);
  };

  const deleteSession = (id: string) => {
    setSessions((prev) => {
      const filtered = prev.filter((s) => s.id !== id);
      if (filtered.length === 0) {
        const session = createSession();
        setActiveSessionId(session.id);
        return [session];
      }
      if (id === activeSessionId) {
        setActiveSessionId(filtered[0].id);
      }
      return filtered;
    });
  };

  return (
    <div className="app">
      <Sidebar
        sessions={sessions}
        activeSessionId={activeSessionId}
        onSelectSession={setActiveSessionId}
        onNewSession={newSession}
        onDeleteSession={deleteSession}
        onOpenSettings={() => setShowSettings(true)}
      />
      <main className="main-content">
        {showMissingKeyBanner && (
          <div className="chat-notice warning">
            <span>Set an LLM API key to start chatting.</span>
            <button className="chat-notice-link" onClick={() => setShowSettings(true)}>
              Open Settings
            </button>
          </div>
        )}
        {showInvalidKeyBanner && (
          <div className="chat-notice error">
            <span>Please enter a correct API key for your LLM.</span>
            <button className="chat-notice-link" onClick={() => setShowSettings(true)}>
              Open Settings
            </button>
          </div>
        )}
        <ChatWindow
          session={activeSession}
          isLoading={isLoading}
          onSendMessage={sendMessage}
        />
      </main>
      {showSettings && (
        <SettingsModal
          settings={settings}
          onSave={async (s) => {
            await invoke("save_settings", {
              request: {
                provider: s.provider,
                openaiKey: s.openaiKey,
                openaiModel: s.openaiModel,
                anthropicKey: s.anthropicKey,
                anthropicModel: s.anthropicModel,
                temperature: s.temperature,
              },
            }).catch(console.error);
            // Reload from backend so keys are re-masked and env flags are fresh.
            invoke<AppSettings>("load_settings")
              .then((fresh) => {
                setSettings(fresh);
                const hasKey = hasConfiguredKey(fresh);
                // Requirement: missing-key notice disappears once a key is saved.
                if (hasKey) {
                  setShowMissingKeyBanner(false);
                  setShowInvalidKeyBanner(false);
                }
              })
              .catch(console.error);
            setShowSettings(false);
          }}
          onClearMemory={async () => {
            await invoke("clear_memory");
          }}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}
