# WhyUIs — AI Roast Chatbot 🔥

> **Ask me anything. I'll roast you first, then actually help.**

A Tauri v2 desktop application that combines brutal comedic roasting with genuine AI assistance. Powered by OpenAI or Anthropic LLMs.

---

## Features

- 🔥 **Roast-first responses** — every answer starts with a personalized roast
- 🧠 **Conversation memory** — remembers your name, preferences, and recurring topics across sessions
- 🔒 **Encrypted local storage** — long-term memory stored encrypted on disk
- 🛡️ **Built-in guardrails** — hardcoded regex patterns block slurs, self-harm, and hate speech
- 🤖 **Multi-provider** — supports OpenAI (GPT-4o) and Anthropic (Claude 3.5 Sonnet)
- 💅 **Dark sci-fi UI** — glass morphism + fire orange accent design
- ⚡ **Tauri v2** — lightweight native desktop app (~5 MB binary)

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 18 + TypeScript + Vite |
| Backend | Rust (Tauri v2) |
| LLM | OpenAI API / Anthropic API |
| Encryption | AES-256-GCM |
| Compression | gzip (flate2) |
| Styling | Pure CSS (no framework) |

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI v2](https://v2.tauri.app/start/prerequisites/)

### Installation

```bash
# Clone the repo
git clone https://github.com/your-org/WhyUIs.git
cd WhyUIs

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Configuration

1. Copy `example.env` to `.env` (for reference — keys are set in the app UI)
2. Launch the app
3. Click ⚙️ **Settings** in the sidebar
4. Enter your OpenAI or Anthropic API key
5. Start getting roasted

---

## Project Structure

```
WhyUIs/
├── src/                      # React + TypeScript frontend
│   ├── components/
│   │   ├── ChatWindow.tsx    # Main chat interface
│   │   ├── MessageBubble.tsx # Message rendering with markdown
│   │   ├── Sidebar.tsx       # Session management
│   │   ├── SettingsModal.tsx # API key + model config
│   │   └── TypingIndicator.tsx
│   ├── styles/               # CSS modules
│   └── types.ts              # TypeScript interfaces
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── main.rs           # Entry point
│   │   ├── lib.rs            # Tauri commands
│   │   ├── chat.rs           # LLM API calls
│   │   ├── memory.rs         # Short + long-term memory
│   │   └── persona.rs        # System prompt + guardrails
│   ├── Cargo.toml
│   └── tauri.conf.json
├── PERSONA.md                # Persona rules documentation
└── example.env               # Environment variable template
```

---

## Guardrails

WhyUIs enforces **non-negotiable content guardrails** in `src-tauri/src/persona.rs`:

- ❌ No racial/ethnic slurs
- ❌ No homophobic, transphobic, or ableist slurs
- ❌ No self-harm encouragement
- ❌ No violence encouragement
- ❌ No targeting of protected classes

These are enforced via regex pattern matching on all LLM output before display. See [PERSONA.md](PERSONA.md) for full documentation.

---

## Building for Production

```bash
npm run tauri build
```

Output bundles are in `src-tauri/target/release/bundle/`.

---

## License

See [LICENSE](LICENSE).
