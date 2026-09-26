import type { Article } from "../../lib/commands";

export const LISTENING_CATEGORIES = [
  "everyday",
  "travelService",
  "developerWork",
  "developerInterview",
  "personal",
] as const;

export type ListeningCategory = typeof LISTENING_CATEGORIES[number];

export function listeningCategory(article: Article | undefined): ListeningCategory {
  if (!article || !article.builtin) return "personal";
  if (article.id.startsWith("builtin.phrases.developer-interview")) return "developerInterview";
  if (article.id.startsWith("builtin.phrases.developer-")) return "developerWork";
  if (["travel", "dining", "hotel", "shopping"].some((scene) => article.id === `builtin.phrases.${scene}`)) {
    return "travelService";
  }
  return "everyday";
}
