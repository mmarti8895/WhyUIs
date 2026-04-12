import { Message } from "../types";
import { User, Bot } from "lucide-react";
import "../styles/messages.css";

interface Props {
  message: Message;
}

function formatContent(content: string): JSX.Element {
  const lines = content.split("\n");
  const elements: JSX.Element[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.startsWith("## ")) {
      elements.push(
        <h2 key={i} className="msg-h2">
          {line.slice(3)}
        </h2>
      );
    } else if (line.startsWith("# ")) {
      elements.push(
        <h1 key={i} className="msg-h1">
          {line.slice(2)}
        </h1>
      );
    } else if (line.startsWith("**") && line.endsWith("**") && line.length > 4) {
      elements.push(
        <p key={i} className="msg-bold-line">
          {line.slice(2, -2)}
        </p>
      );
    } else if (line.startsWith("- ") || line.startsWith("• ")) {
      const bulletText = line.startsWith("- ") ? line.slice(2) : line.slice(2);
      elements.push(
        <li key={i} className="msg-bullet">
          {formatInline(bulletText)}
        </li>
      );
    } else if (line.trim() === "") {
      elements.push(<div key={i} className="msg-spacer" />);
    } else if (line.startsWith("⚠️") || line.startsWith("🔥") || line.startsWith("✅")) {
      elements.push(
        <p key={i} className="msg-highlight">
          {formatInline(line)}
        </p>
      );
    } else {
      elements.push(
        <p key={i} className="msg-paragraph">
          {formatInline(line)}
        </p>
      );
    }
    i++;
  }

  return <div className="msg-content">{elements}</div>;
}

function formatInline(text: string): React.ReactNode {
  const parts = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g);
  return parts.map((part, i) => {
    if (part.startsWith("**") && part.endsWith("**")) {
      return <strong key={i}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return <code key={i} className="inline-code">{part.slice(1, -1)}</code>;
    }
    return part;
  });
}

export default function MessageBubble({ message }: Props) {
  const isUser = message.role === "user";
  const time = new Date(message.timestamp).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });

  return (
    <div className={`message-wrapper ${isUser ? "user" : "assistant"}`}>
      <div className="message-avatar">
        {isUser ? <User size={16} /> : <Bot size={16} />}
      </div>
      <div className="message-bubble">
        <div className="message-role">{isUser ? "You" : "RoastBot 🔥"}</div>
        {isUser ? (
          <p className="message-text">{message.content}</p>
        ) : (
          formatContent(message.content)
        )}
        <div className="message-time">{time}</div>
      </div>
    </div>
  );
}
