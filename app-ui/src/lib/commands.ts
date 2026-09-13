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
  | { type: "custom"; senseUids: string[] };

export interface CollectionSession {
  collectionId: string;
  sessionId: string;
  datasetId?: string;
  collectionType: string;
  totalCount: number;
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

export type UiLanguage = "system" | "en" | "zh-CN";

export interface SettingsDto {
  uiLanguage: UiLanguage;
}

export const getAppInfo = () => invoke<AppInfo>("get_app_info");
export const listDatasets = () => invoke<DatasetSummary[]>("list_datasets");
export const getHome = (datasetId: string) =>
  invoke<HomeDto>("get_home", { datasetId });
export const startCollection = (spec: CollectionSpec) =>
  invoke<CollectionSession>("start_collection", { spec });
export const nextQuestion = (collectionId: string) =>
  invoke<QuizQuestion | null>("next_question", { collectionId });
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
export const speak = (text: string, locale = "en-US", rate = 1) =>
  invoke<void>("speak", { request: { text, locale, rate } });
export const stopSpeech = () => invoke<void>("stop_speech");
export const createDataset = (name: string) =>
  invoke<DatasetSummary>("create_dataset", { name });
export const renameDataset = (datasetId: string, name: string) =>
  invoke<void>("rename_dataset", { datasetId, name });
export const deleteDataset = (datasetId: string) =>
  invoke<void>("delete_dataset", { datasetId });
export const previewDatasetCsv = (path: string) =>
  invoke<CsvImportPreview>("preview_dataset_csv", { path });
export const importDatasetCsv = (datasetId: string, path: string) =>
  invoke<CsvImportResult>("import_dataset_csv", { datasetId, path });
export const getSettings = () => invoke<SettingsDto>("get_settings");
export const updateSettings = (settings: SettingsDto) =>
  invoke<void>("update_settings", { settings });
