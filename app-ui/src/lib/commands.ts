import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  name: string;
  knowledgeModule: string;
  version: string;
}

export interface DatasetSummary {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
  preloaded: boolean;
  wordCount: number;
}

export interface DailyActivity {
  localDate: string;
  attemptCount: number;
  correctCount: number;
  wrongCount: number;
}

export interface HomeDto {
  dataset: DatasetSummary;
  progress: { total: number; answered: number; unseen: number };
  totals: { explored: number; correct: number; mistakes: number };
  dailyActivity: DailyActivity[];
  mistakeBuckets: {
    atLeastOne: number;
    atLeastTwo: number;
    atLeastThree: number;
    atLeastFive: number;
    lastWrong: number;
  };
}

export type CollectionSpec =
  | { type: "dataset"; datasetId: string }
  | { type: "unseen"; datasetId: string }
  | { type: "answered"; datasetId?: string }
  | { type: "correct"; datasetId?: string }
  | { type: "wrong"; datasetId?: string; minWrongCount: number }
  | { type: "lastWrong"; datasetId?: string }
  | { type: "custom"; senseUids: string[] }
  | { type: "myVocabulary" };

export interface CollectionSession {
  collectionId: string;
  sessionId: string;
  datasetId?: string;
  collectionType: string;
  totalCount: number;
  answeredCount: number;
  correctCount: number;
  wrongCount: number;
  newWordCount: number;
  wrongSenseUids: string[];
}

export interface QuestionOption {
  optionId: string;
  senseUid: string;
  lemma: string;
}

export interface QuizQuestion {
  questionId: string;
  senseUid: string;
  promptZh: string;
  options: QuestionOption[];
  answeredCount: number;
  totalCount: number;
}

export interface AnswerResult {
  correct: boolean;
  wasNew: boolean;
  correctSenseUid: string;
  selectedSenseUid: string;
  lemma: string;
  ipa: string;
  zhGloss: string;
  exampleEn: string;
  exampleZh: string;
}

export interface WrongWord {
  senseUid: string;
  lemma: string;
  ipa: string;
  zhGloss: string;
  wrongCount: number;
  correctCount: number;
  lastResult: string;
}

export interface LexiconEntry {
  senseUid: string;
  lemma: string;
  ipa: string;
  partOfSpeech: string;
  zhGloss: string;
  exampleEn: string;
  datasetNames: string[];
  attemptCount: number;
  correctCount: number;
  wrongCount: number;
  inMyVocabulary: boolean;
}

export interface Article {
  id: string;
  title: string;
  body: string;
  translatedBody: string | null;
  createdAt: number;
}

export interface CsvImportPreview {
  fileName: string;
  totalRows: number;
  validRows: number;
  issues: Array<{ row: number; message: string }>;
  sample: Array<{
    senseUid: string;
    lemma: string;
    partOfSpeech: string;
    quizPromptZh: string;
  }>;
}

export interface CsvImportResult {
  importedItems: number;
  insertedSenses: number;
  updatedSenses: number;
}

export type DatasetImportStrategy = "addOnly" | "updateExisting" | "replaceDataset";

export type UiLanguage = "system" | "en" | "zh-CN";
export type UiTheme = "system" | "light" | "dark";
export type SpeechLocale = "en-US" | "en-GB";
export const SPEECH_VOICES = ["male", "female", "american", "british", "hong-kong", "indian", "japanese"] as const;
export type SpeechVoice = typeof SPEECH_VOICES[number];

export interface SettingsDto {
  speechLocale: SpeechLocale;
  speechRatePercent: number;
  speechVoice: SpeechVoice;
  uiLanguage: UiLanguage;
  uiTheme: UiTheme;
}

export interface BackupStatus {
  lastBackupAt?: string;
}

export interface BackupVersion {
  id: string;
  createdAt: string;
  sizeBytes: number;
  reason: "automatic" | "preRestore";
}

export interface RestoreResult {
  automaticBackupPath: string;
}

export const getAppInfo = () => invoke<AppInfo>("get_app_info");
export const listDatasets = () => invoke<DatasetSummary[]>("list_datasets");
export const getHome = (datasetId: string) =>
  invoke<HomeDto>("get_home", { datasetId });
export const startCollection = (spec: CollectionSpec, limit?: number) =>
  invoke<CollectionSession>("start_collection", { spec, limit });
export const nextQuestion = (collectionId: string) =>
  invoke<QuizQuestion | null>("next_question", { collectionId });
export const getResumableSession = () =>
  invoke<CollectionSession | null>("get_resumable_session");
export const submitAnswer = (
  collectionId: string,
  questionId: string,
  selectedOptionId: string,
  latencyMs?: number,
) =>
  invoke<AnswerResult>("submit_answer", {
    collectionId,
    questionId,
    selectedOptionId,
    latencyMs,
  });
export const finishSession = (sessionId: string) =>
  invoke<void>("finish_session", { sessionId });
export const listWrongWords = (
  datasetId: string | undefined,
  minWrongCount: number,
  lastWrongOnly: boolean,
) =>
  invoke<WrongWord[]>("list_wrong_words", {
    datasetId,
    minWrongCount,
    lastWrongOnly,
  });
export const searchLexicon = (query: string) =>
  invoke<LexiconEntry[]>("search_lexicon", { query });
export const addToMyVocabulary = (senseUid: string) =>
  invoke<void>("add_to_my_vocabulary", { senseUid });
export const removeFromMyVocabulary = (senseUid: string) =>
  invoke<void>("remove_from_my_vocabulary", { senseUid });
export const getBackupStatus = () => invoke<BackupStatus>("get_backup_status");
export const ensureAutomaticBackup = () =>
  invoke<BackupVersion[]>("ensure_automatic_backup");
export const listBackupVersions = () =>
  invoke<BackupVersion[]>("list_backup_versions");
export const exportBackup = (path: string) => invoke<BackupStatus>("export_backup", { path });
export const importBackup = (path: string) => invoke<RestoreResult>("import_backup", { path });
export const restoreBackupVersion = (id: string) =>
  invoke<RestoreResult>("restore_backup_version", { id });
export const speak = (text: string, locale = "en-US", rate = 1, voice: SpeechVoice = "female") =>
  invoke<void>("speak", { request: { text, locale, rate, voice } });
export const pauseSpeech = () => invoke<void>("pause_speech");
export const resumeSpeech = () => invoke<void>("resume_speech");
export const stopSpeech = () => invoke<void>("stop_speech");
export const listArticles = () => invoke<Article[]>("list_articles");
export const importArticle = (path: string) => invoke<Article>("import_article", { path });
export const saveArticleTranslation = (articleId: string, translatedBody: string) =>
  invoke<void>("save_article_translation", { articleId, translatedBody });
export const deleteArticle = (articleId: string) => invoke<void>("delete_article", { articleId });
export const createDataset = (name: string) =>
  invoke<DatasetSummary>("create_dataset", { name });
export const renameDataset = (datasetId: string, name: string) =>
  invoke<void>("rename_dataset", { datasetId, name });
export const reorderDatasets = (datasetIds: string[]) =>
  invoke<void>("reorder_datasets", { datasetIds });
export const deleteDataset = (datasetId: string) =>
  invoke<void>("delete_dataset", { datasetId });
export const previewDatasetCsv = (path: string) =>
  invoke<CsvImportPreview>("preview_dataset_csv", { path });
export const importDatasetCsv = (datasetId: string, path: string, strategy: DatasetImportStrategy) =>
  invoke<CsvImportResult>("import_dataset_csv", { datasetId, path, strategy });
export const exportDatasetCsv = (datasetId: string, path: string) =>
  invoke<number>("export_dataset_csv", { datasetId, path });
export const getSettings = () => invoke<SettingsDto>("get_settings");
export const updateSettings = (settings: SettingsDto) =>
  invoke<void>("update_settings", { settings });
