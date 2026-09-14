import { createContext, useContext, useEffect, useLayoutEffect, useMemo, useState, type ReactNode } from "react";

import en from "../locales/en.json";
import zhCn from "../locales/zh-CN.json";
import { getSettings, updateSettings, type SpeechLocale, type UiLanguage, type UiTheme } from "./commands";

type Parameters = Record<string, string | number>;
type Translator = (key: string, parameters?: Parameters) => string;

interface I18nValue {
  language: UiLanguage;
  resolvedLanguage: "en" | "zh-CN";
  resolvedTheme: "light" | "dark";
  setLanguage: (language: UiLanguage) => Promise<void>;
  setSpeechLocale: (locale: SpeechLocale) => Promise<void>;
  setSpeechRatePercent: (rate: number) => Promise<void>;
  setTheme: (theme: UiTheme) => Promise<void>;
  speechLocale: SpeechLocale;
  speechRatePercent: number;
  theme: UiTheme;
  t: Translator;
}

const resources: Record<"en" | "zh-CN", Record<string, string>> = {
  en,
  "zh-CN": zhCn,
};

const I18nContext = createContext<I18nValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<UiLanguage>("system");
  const [theme, setThemeState] = useState<UiTheme>("system");
  const [speechLocale, setSpeechLocaleState] = useState<SpeechLocale>("en-US");
  const [speechRatePercent, setSpeechRatePercentState] = useState(100);
  const [, setSystemPreferenceRevision] = useState(0);
  const resolvedLanguage = resolveLanguage(language);
  const resolvedTheme = resolveTheme(theme);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getSettings().then((settings) => {
      setLanguageState(settings.uiLanguage);
      setThemeState(settings.uiTheme);
      setSpeechLocaleState(settings.speechLocale);
      setSpeechRatePercentState(settings.speechRatePercent);
    }).catch(() => undefined);
  }, []);

  useEffect(() => {
    document.documentElement.lang = resolvedLanguage;
  }, [resolvedLanguage]);

  useLayoutEffect(() => {
    applyDocumentTheme(resolvedTheme);
  }, [resolvedTheme]);

  useEffect(() => {
    const handleSystemPreferenceChange = () => setSystemPreferenceRevision((revision) => revision + 1);
    const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
    window.addEventListener("languagechange", handleSystemPreferenceChange);
    colorScheme.addEventListener("change", handleSystemPreferenceChange);
    return () => {
      window.removeEventListener("languagechange", handleSystemPreferenceChange);
      colorScheme.removeEventListener("change", handleSystemPreferenceChange);
    };
  }, []);

  const value = useMemo<I18nValue>(() => ({
    language,
    resolvedLanguage,
    resolvedTheme,
    setLanguage: async (nextLanguage) => {
      setLanguageState(nextLanguage);
      if ("__TAURI_INTERNALS__" in window) {
        await updateSettings({ speechLocale, speechRatePercent, uiLanguage: nextLanguage, uiTheme: theme });
      }
    },
    setSpeechLocale: async (nextLocale) => {
      setSpeechLocaleState(nextLocale);
      if ("__TAURI_INTERNALS__" in window) {
        await updateSettings({ speechLocale: nextLocale, speechRatePercent, uiLanguage: language, uiTheme: theme });
      }
    },
    setSpeechRatePercent: async (nextRate) => {
      setSpeechRatePercentState(nextRate);
      if ("__TAURI_INTERNALS__" in window) {
        await updateSettings({ speechLocale, speechRatePercent: nextRate, uiLanguage: language, uiTheme: theme });
      }
    },
    setTheme: async (nextTheme) => {
      setThemeState(nextTheme);
      if ("__TAURI_INTERNALS__" in window) {
        await updateSettings({ speechLocale, speechRatePercent, uiLanguage: language, uiTheme: nextTheme });
      }
    },
    speechLocale,
    speechRatePercent,
    theme,
    t: (key, parameters) => translate(resources[resolvedLanguage], key, parameters),
  }), [language, resolvedLanguage, resolvedTheme, speechLocale, speechRatePercent, theme]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

function resolveTheme(theme: UiTheme): "light" | "dark" {
  if (theme !== "system") return theme;
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

export function initializeSystemTheme(): void {
  applyDocumentTheme(resolveTheme("system"));
}

function applyDocumentTheme(theme: "light" | "dark"): void {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
}

export function useI18n(): I18nValue {
  const context = useContext(I18nContext);
  if (!context) throw new Error("useI18n must be used within I18nProvider");
  return context;
}

function resolveLanguage(language: UiLanguage): "en" | "zh-CN" {
  if (language !== "system") return language;
  const systemLanguage = typeof navigator === "undefined" ? "en" : navigator.language;
  return systemLanguage.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

function translate(resource: Record<string, string>, key: string, parameters?: Parameters): string {
  const template = resource[key] ?? resources.en[key] ?? key;
  return Object.entries(parameters ?? {}).reduce(
    (result, [name, value]) => result.replaceAll(`{${name}}`, String(value)),
    template,
  );
}
