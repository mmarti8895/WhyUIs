import "../styles/messages.css";
import BrandFireIcon from "./BrandFireIcon";

export default function TypingIndicator() {
  return (
    <div className="message-wrapper assistant">
      <div className="message-avatar typing-avatar">
        <BrandFireIcon size="sm" />
      </div>
      <div className="message-bubble typing-bubble">
        <div className="message-role">RoastBot</div>
        <div className="typing-dots">
          <span />
          <span />
          <span />
        </div>
      </div>
    </div>
  );
}
