import { useState, useCallback } from "react";
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
  openaiModel: "gpt-4o",
  anthropicKey: "",
  anthropicModel: "claude-3-5-sonnet-20241022",
  temperature: 0.9,
};

export default function App() {
  const [sessions, setSessions] = useState<ChatSession[]>([createSession()]);
  const [activeSessionId, setActiveSessionId] = useState<string>(sessions[0].id);
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [showSettings, setShowSettings] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

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
          settings: {
            provider: settings.provider,
            openai_key: settings.openaiKey,
            openai_model: settings.openaiModel,
            anthropic_key: settings.anthropicKey,
            anthropic_model: settings.anthropicModel,
            temperature: settings.temperature,
          },
          history: activeSession.messages.slice(-14).map((m) => ({
            role: m.role,
            content: m.content,
          })),
        });

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
        const errorMessage: Message = {
          id: crypto.randomUUID(),
          role: "assistant",
          content: `⚠️ **Error:** ${err}\n\nConfigure your API keys in Settings (⚙️) to get roasted properly.`,
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
        <ChatWindow
          session={activeSession}
          isLoading={isLoading}
          onSendMessage={sendMessage}
        />
      </main>
      {showSettings && (
        <SettingsModal
          settings={settings}
          onSave={(s) => {
            setSettings(s);
            setShowSettings(false);
          }}
          onClose={() => setShowSettings(false)}
        />
      )}
    </div>
  );
}
