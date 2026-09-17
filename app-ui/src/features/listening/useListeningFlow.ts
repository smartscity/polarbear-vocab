import { useCallback, useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  deleteArticle,
  importArticle,
  listArticles,
  pauseSpeech,
  resumeSpeech,
  saveArticleTranslation,
  speak,
  stopSpeech,
  type Article,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import { translateEnglishMarkdown } from "./articleTranslation";

export type PlaybackState = "idle" | "paused" | "playing";
export type ArticleImportPhase = "choosing" | "importing";

export function useListeningFlow(onError: (error: unknown) => void) {
  const {
    setSpeechRatePercent,
    setSpeechVoice,
    speechLocale,
    speechRatePercent,
    speechVoice,
  } = useI18n();
  const [articles, setArticles] = useState<Article[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [playback, setPlayback] = useState<PlaybackState>("idle");
  const [importPhase, setImportPhase] = useState<ArticleImportPhase | null>(null);
  const [translating, setTranslating] = useState(false);
  const selected = useMemo(
    () => articles.find((article) => article.id === selectedId) ?? articles[0],
    [articles, selectedId],
  );

  const refresh = useCallback(async (preferredId?: string) => {
    const available = await listArticles();
    setArticles(available);
    setSelectedId(available.some((article) => article.id === preferredId) ? preferredId ?? "" : available[0]?.id ?? "");
  }, []);

  useEffect(() => {
    if ("__TAURI_INTERNALS__" in window) void refresh().catch(onError);
  }, [onError, refresh]);

  const chooseFile = async () => {
    if (importPhase) return;
    setImportPhase("choosing");
    try {
      const path = await open({
        directory: false,
        fileAccessMode: "copy",
        filters: [{ name: "Text article", extensions: ["txt", "md"] }],
        multiple: false,
        pickerMode: "document",
      });
      if (typeof path !== "string") return;
      setImportPhase("importing");
      const imported = await importArticle(path);
      await refresh(imported.id);
    } catch (error) {
      onError(error);
    } finally {
      setImportPhase(null);
    }
  };

  const play = async () => {
    if (!selected) return;
    try {
      await speak(selected.body, speechLocale, speechRatePercent / 100, speechVoice);
      setPlayback("playing");
    } catch (error) {
      onError(error);
    }
  };

  const pause = async () => {
    try {
      await pauseSpeech();
      setPlayback("paused");
    } catch (error) {
      onError(error);
    }
  };

  const resume = async () => {
    try {
      await resumeSpeech();
      setPlayback("playing");
    } catch (error) {
      onError(error);
    }
  };

  const stop = async () => {
    try {
      await stopSpeech();
      setPlayback("idle");
    } catch (error) {
      onError(error);
    }
  };

  const remove = async () => {
    if (!selected) return;
    try {
      await stopSpeech();
      await deleteArticle(selected.id);
      setPlayback("idle");
      await refresh();
    } catch (error) {
      onError(error);
    }
  };

  const changeRate = async (rate: number) => {
    try {
      if (playback !== "idle") await stopSpeech();
      setPlayback("idle");
      await setSpeechRatePercent(rate);
    } catch (error) {
      onError(error);
    }
  };

  const changeVoice = async (voice: Parameters<typeof setSpeechVoice>[0]) => {
    try {
      if (playback !== "idle") await stopSpeech();
      setPlayback("idle");
      await setSpeechVoice(voice);
    } catch (error) {
      onError(error);
    }
  };

  const select = async (articleId: string) => {
    try {
      if (playback !== "idle") await stopSpeech();
      setPlayback("idle");
      setSelectedId(articleId);
    } catch (error) {
      onError(error);
    }
  };

  const translate = async () => {
    if (!selected || translating) return;
    const articleId = selected.id;
    setTranslating(true);
    try {
      const translatedBody = await translateEnglishMarkdown(selected.body);
      await saveArticleTranslation(articleId, translatedBody);
      setArticles((current) => current.map((article) => (
        article.id === articleId ? { ...article, translatedBody } : article
      )));
    } catch (error) {
      onError(error);
    } finally {
      setTranslating(false);
    }
  };

  return {
    articles,
    changeRate,
    changeVoice,
    chooseFile,
    importPhase,
    pause,
    playback,
    play,
    remove,
    resume,
    select,
    selected,
    speechRatePercent,
    speechVoice,
    stop,
    translate,
    translating,
  };
}
