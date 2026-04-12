export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: Message[];
  createdAt: number;
}

export interface AppSettings {
  provider: "openai" | "anthropic";
  openaiKey: string;
  openaiModel: string;
  anthropicKey: string;
  anthropicModel: string;
  temperature: number;
  /** True when this provider's config is locked by the .env file. */
  openaiFromEnv: boolean;
  anthropicFromEnv: boolean;
}
