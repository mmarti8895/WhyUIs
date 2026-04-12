# WhyUIs — AI Roast Chatbot

Ask anything, get roasted first, then get a real answer.

Desktop app built with React + TypeScript + Tauri v2 (Rust backend), supporting OpenAI and Anthropic models.

---

## The App in Action

![WhyUIs app in action](src-tauri/icons/app-in-action.png)

## This App is Rude AF

![WhyUIs is rude AF](src-tauri/icons/is-rude-af.png)

## Features

- Roast-first assistant with a structured answer phase
- Multi-session chat UI
- Local memory (short-term + long-term)
- Guardrail sanitization on model output
- API key management in-app
- `.env` fallback support for API keys and model defaults

---

## Setup Workflows

### Prerequisites

- Node.js 18+
- Rust stable toolchain
- Tauri v2 prerequisites for your OS

### Install

```bash
npm install
```

### Workflow A: In-app key setup (recommended)

1. Start the app: `npm run tauri dev`
2. Open Settings from the sidebar
3. Enter OpenAI and/or Anthropic API key
4. Save Changes

If no valid key is configured, the chat pane shows a top warning banner with a link to open Settings.

### Workflow B: `.env` defaults

1. Copy `example.env` to `.env`
2. Fill provider keys/models in `.env`
3. Start app: `npm run tauri dev`

Important key handling rules:

- Blank keys are treated as not set
- Placeholder keys are treated as not set:
	- `sk-your-openai-key-here`
	- `sk-ant-your-anthropic-key-here`
- If `.env` key is not set, user can set key in-app and it is persisted locally

Provider preference rule:

- If both providers are configured with valid keys, OpenAI is preferred by default

---

## Running the App

### Development

```bash
npm run tauri dev
```

### Frontend-only build

```bash
npm run build
```

### Production build

```bash
npm run tauri build
```

Output bundles: `src-tauri/target/release/bundle/`

---

## API Key Security and Encryption Policy

### At rest

- API settings are stored in an encrypted local file:
	- `%LOCALAPPDATA%/WhyUIs/settings.dat`
- Conversation memory is also encrypted locally:
	- `%LOCALAPPDATA%/WhyUIs/memory.dat`
- Encryption algorithm: AES-256-GCM
- Nonce is generated per encryption operation and prepended to ciphertext

### In transit

- Model requests are sent over HTTPS using `reqwest` with `rustls-tls`

### Enforcement behavior

- Frontend does not receive raw API keys from backend
- UI gets masked key sentinel (`••••••`) when a key exists
- Placeholder/blank `.env` keys are normalized to unset
- Missing-key and invalid-key states trigger top-of-chat notices with a Settings shortcut

---

## Current Test Coverage

Current backend test run status: passing.

Command:

```bash
cd src-tauri
cargo test
```

Current totals:

- 14 unit tests passed
- 0 failed

Covered areas:

- Chat provider error behavior when keys are missing
- Memory behavior (compaction, extraction, encryption/decryption)
- Persona guardrails and sanitize behavior

---

## Notable Recent Updates

- Added encrypted local settings store for API credentials
- Added `.env` fallback handling with placeholder-key detection
- Added top-of-chat UX notices for missing/invalid keys
- Updated model dropdown handling so selected model values are reflected correctly
- Fixed stale Rust chat tests after settings type refactor

---

## Project Structure

```
WhyUIs/
├── src/
│   ├── App.tsx
│   ├── components/
│   ├── styles/
│   └── types.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── chat.rs
│   │   ├── lib.rs
│   │   ├── memory.rs
│   │   ├── persona.rs
│   │   └── settings.rs
│   └── tauri.conf.json
├── PERSONA.md
├── example.env
└── README.md
```

---

## License

MIT — see [LICENSE](LICENSE).
