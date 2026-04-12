import { ChatSession } from "../types";
import { Plus, Trash2, Settings, Flame, MessageSquare } from "lucide-react";
import "../styles/sidebar.css";

interface Props {
  sessions: ChatSession[];
  activeSessionId: string;
  onSelectSession: (id: string) => void;
  onNewSession: () => void;
  onDeleteSession: (id: string) => void;
  onOpenSettings: () => void;
}

export default function Sidebar({
  sessions,
  activeSessionId,
  onSelectSession,
  onNewSession,
  onDeleteSession,
  onOpenSettings,
}: Props) {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <div className="sidebar-logo">
          <Flame className="logo-flame" size={22} />
          <span className="logo-text">WhyUIs</span>
        </div>
        <button className="new-chat-btn" onClick={onNewSession} title="New chat (Ctrl+N)">
          <Plus size={18} />
        </button>
      </div>

      <div className="sidebar-label">ROAST HISTORY</div>

      <nav className="sessions-list">
        {sessions.map((session) => (
          <div
            key={session.id}
            className={`session-item ${session.id === activeSessionId ? "active" : ""}`}
            onClick={() => onSelectSession(session.id)}
          >
            <MessageSquare size={14} className="session-icon" />
            <span className="session-title">{session.title}</span>
            <button
              className="session-delete"
              onClick={(e) => {
                e.stopPropagation();
                onDeleteSession(session.id);
              }}
              title="Delete session"
            >
              <Trash2 size={13} />
            </button>
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        <button className="settings-btn" onClick={onOpenSettings}>
          <Settings size={16} />
          <span>Settings</span>
        </button>
        <div className="sidebar-version">v1.0.0</div>
      </div>
    </aside>
  );
}
