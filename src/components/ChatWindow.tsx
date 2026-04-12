import { useState, useRef, useEffect, KeyboardEvent } from "react";
import { ChatSession } from "../types";
import MessageBubble from "./MessageBubble";
import TypingIndicator from "./TypingIndicator";
import { Send, Flame } from "lucide-react";
import "../styles/chat.css";

interface Props {
  session: ChatSession;
  isLoading: boolean;
  onSendMessage: (message: string) => void;
}

export default function ChatWindow({ session, isLoading, onSendMessage }: Props) {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [session.messages, isLoading]);

  const handleSend = () => {
    if (input.trim() && !isLoading) {
      onSendMessage(input.trim());
      setInput("");
      if (textareaRef.current) {
        textareaRef.current.style.height = "auto";
      }
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    e.target.style.height = "auto";
    e.target.style.height = `${Math.min(e.target.scrollHeight, 200)}px`;
  };

  const isEmpty = session.messages.length === 0;

  return (
    <div className="chat-window">
      <div className="chat-header">
        <div className="chat-header-left">
          <Flame className="flame-icon" size={20} />
          <span className="chat-title">{session.title}</span>
        </div>
        <div className="chat-header-right">
          <span className="model-badge">ROAST MODE</span>
        </div>
      </div>

      <div className="messages-container">
        {isEmpty ? (
          <div className="empty-state">
            <div className="empty-icon">🔥</div>
            <h2 className="empty-title">Ask me anything.</h2>
            <p className="empty-subtitle">
              I'll roast you first, then actually help.
              <br />
              <span className="empty-hint">Press Enter to send — Shift+Enter for new line</span>
            </p>
            <div className="empty-examples">
              <button
                className="example-chip"
                onClick={() => onSendMessage("How do I center a div in CSS?")}
              >
                How do I center a div in CSS?
              </button>
              <button
                className="example-chip"
                onClick={() => onSendMessage("What's the best programming language?")}
              >
                What's the best programming language?
              </button>
              <button
                className="example-chip"
                onClick={() => onSendMessage("Help me fix my life")}
              >
                Help me fix my life
              </button>
            </div>
          </div>
        ) : (
          <div className="messages-list">
            {session.messages.map((msg) => (
              <MessageBubble key={msg.id} message={msg} />
            ))}
            {isLoading && <TypingIndicator />}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      <div className="input-area">
        <div className="input-container">
          <textarea
            ref={textareaRef}
            className="message-input"
            value={input}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder="Ask something... if you dare 🔥"
            rows={1}
            disabled={isLoading}
          />
          <button
            className={`send-button ${input.trim() && !isLoading ? "active" : ""}`}
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            aria-label="Send message"
          >
            <Send size={18} />
          </button>
        </div>
        <p className="input-hint">Enter to send · Shift+Enter for new line</p>
      </div>
    </div>
  );
}
