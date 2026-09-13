import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from "react";

import en from "../locales/en.json";
import zhCn from "../locales/zh-CN.json";
import { getSettings, updateSettings, type UiLanguage } from "./commands";

type Parameters = Record<string, string | number>;
type Translator = (key: string, parameters?: Parameters) => string;

interface I18nValue {
  language: UiLanguage;
  resolvedLanguage: "en" | "zh-CN";
  setLanguage: (language: UiLanguage) => Promise<void>;
  t: Translator;
}

const resources: Record<"en" | "zh-CN", Record<string, string>> = {
  en,
  "zh-CN": zhCn,
};

const I18nContext = createContext<I18nValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<UiLanguage>("system");
  const [, setSystemLanguageRevision] = useState(0);
  const resolvedLanguage = resolveLanguage(language);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getSettings().then((settings) => setLanguageState(settings.uiLanguage));
  }, []);

  useEffect(() => {
    document.documentElement.lang = resolvedLanguage;
  }, [resolvedLanguage]);

  useEffect(() => {
    const handleLanguageChange = () => setSystemLanguageRevision((revision) => revision + 1);
    window.addEventListener("languagechange", handleLanguageChange);
    return () => window.removeEventListener("languagechange", handleLanguageChange);
  }, []);

  const value = useMemo<I18nValue>(() => ({
    language,
    resolvedLanguage,
    setLanguage: async (nextLanguage) => {
      setLanguageState(nextLanguage);
      if ("__TAURI_INTERNALS__" in window) {
        await updateSettings({ uiLanguage: nextLanguage });
      }
    },
    t: (key, parameters) => translate(resources[resolvedLanguage], key, parameters),
  }), [language, resolvedLanguage]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
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
