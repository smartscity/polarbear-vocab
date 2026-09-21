import type { Meta, StoryObj } from "@storybook/react-vite";

import { DatasetsView } from "../features/datasets/DatasetsView";
import { HomeView } from "../features/home/HomeView";
import { LexiconView } from "../features/lexicon/LexiconView";
import { ListeningView } from "../features/listening/ListeningView";
import { MistakesView } from "../features/mistakes/MistakesView";
import { SettingsView } from "../features/settings/SettingsView";
import { SessionSetupView } from "../features/study/SessionSetupView";
import { StudyView } from "../features/study/StudyView";
import { AppShell, type NavScreen } from "../layout/AppShell";
import type { AnswerResult as AnswerResultDto, Article, DatasetSummary, HomeDto, LexiconEntry, QuizQuestion, WrongWord } from "../lib/commands";

const datasets: DatasetSummary[] = [
  { id: "core", name: "Everyday English", createdAt: 1, updatedAt: 1, preloaded: true, wordCount: 640 },
  { id: "travel", name: "Travel notes", createdAt: 2, updatedAt: 2, preloaded: false, wordCount: 84 },
  { id: "work", name: "Product vocabulary", createdAt: 3, updatedAt: 3, preloaded: false, wordCount: 126 },
];

const activity = Array.from({ length: 30 }, (_, index) => ({
  localDate: `2026-08-${String(index + 1).padStart(2, "0")}`,
  attemptCount: [0, 3, 8, 1, 12, 5, 0][index % 7],
  correctCount: [0, 2, 7, 1, 9, 4, 0][index % 7],
  wrongCount: [0, 1, 1, 0, 3, 1, 0][index % 7],
}));

const home: HomeDto = {
  dataset: datasets[0],
  progress: { total: 640, answered: 238, unseen: 402 },
  totals: { explored: 238, correct: 187, mistakes: 51 },
  dailyActivity: activity,
  mistakeBuckets: { atLeastOne: 51, atLeastTwo: 23, atLeastThree: 12, atLeastFive: 4, lastWrong: 18 },
};

const question: QuizQuestion = {
  questionId: "question-1",
  senseUid: "sense-resilient",
  promptZh: "能够迅速恢复的；有韧性的",
  answeredCount: 7,
  totalCount: 24,
  options: [
    { optionId: "a", senseUid: "sense-resilient", lemma: "resilient" },
    { optionId: "b", senseUid: "sense-reluctant", lemma: "reluctant" },
    { optionId: "c", senseUid: "sense-relevant", lemma: "relevant" },
    { optionId: "d", senseUid: "sense-reliable", lemma: "reliable" },
  ],
};

const answer: AnswerResultDto = {
  correct: false,
  wasNew: true,
  correctSenseUid: "sense-resilient",
  selectedSenseUid: "sense-reluctant",
  lemma: "resilient",
  ipa: "/rɪˈzɪliənt/",
  zhGloss: "有韧性的；能迅速恢复的",
  exampleEn: "Small teams are often surprisingly resilient.",
  exampleZh: "小团队往往有出人意料的韧性。",
};

const words: WrongWord[] = [
  { senseUid: "sense-resilient", lemma: "resilient", ipa: "/rɪˈzɪliənt/", zhGloss: "有韧性的", wrongCount: 4, correctCount: 2, lastResult: "wrong" },
  { senseUid: "sense-subtle", lemma: "subtle", ipa: "/ˈsʌtl/", zhGloss: "微妙的", wrongCount: 3, correctCount: 1, lastResult: "correct" },
  { senseUid: "sense-concise", lemma: "concise", ipa: "/kənˈsaɪs/", zhGloss: "简洁的", wrongCount: 2, correctCount: 3, lastResult: "wrong" },
];

const lexiconResults: LexiconEntry[] = [{
  senseUid: "earn.v.01",
  lemma: "earn",
  ipa: "/ɜːrn/",
  partOfSpeech: "verb",
  zhGloss: "获得（通过努力赢得）；赚得",
  exampleEn: "She earned their trust.",
  datasetNames: ["CET-4", "CET-6", "IELTS"],
  attemptCount: 8,
  correctCount: 6,
  wrongCount: 2,
  inMyVocabulary: false,
}];

const articles: Article[] = [
  {
    id: "article-1",
    title: "A Walk Through the Rain",
    body: "## A quiet morning\n\nThe rain began just after **breakfast**. Maya opened her umbrella and walked toward the station.\n\n> She slowed down to listen to the rhythm of the drops.",
    translatedBody: "## 一个安静的早晨\n\n雨在**早餐**后不久开始下。玛雅撑开雨伞，向车站走去。\n\n> 她放慢脚步，聆听雨滴的节奏。",
    createdAt: 1,
    builtin: true,
  },
  {
    id: "article-2",
    title: "Small Habits",
    body: "Small habits become strong routines when we repeat them with care.",
    translatedBody: null,
    createdAt: 2,
    builtin: false,
  },
];

function Screen(props: { children: React.ReactNode; screen: NavScreen | "session" | "study" }) {
  return <AppShell appName="Polarbear Vocab" footer="Copyright © 2020-2026 smartscity All rights reserved." onNavigate={() => undefined} screen={props.screen}>{props.children}</AppShell>;
}

const noOp = () => undefined;
const meta = {
  title: "Screens",
  parameters: { controls: { disable: true } },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const Home: Story = { render: () => <Screen screen="home"><HomeView datasets={datasets} home={home} onContinue={noOp} onDatasetChange={noOp} onMistakes={noOp} onPracticeCollection={noOp} onResume={noOp} resumableCount={12} /></Screen> };

export const DatasetList: Story = { render: () => <Screen screen="datasets"><DatasetsView datasets={datasets} onChanged={async () => undefined} onError={noOp} onSelect={noOp} onStart={noOp} selectedDatasetId="" /></Screen> };

export const DatasetDetail: Story = { render: () => <Screen screen="datasets"><DatasetsView datasets={datasets} onChanged={async () => undefined} onError={noOp} onSelect={noOp} onStart={noOp} selectedDatasetId="core" /></Screen> };

export const Lexicon: Story = { render: () => <Screen screen="lexicon"><LexiconView busy={false} onPractice={noOp} onQueryChange={noOp} onSearch={noOp} onSpeak={noOp} onToggleVocabulary={noOp} query="earn" results={lexiconResults} searched /></Screen> };

export const Listening: Story = { render: () => <Screen screen="listening"><ListeningView articles={articles} importPhase={null} onDelete={noOp} onError={noOp} onImport={noOp} onPause={noOp} onPlay={noOp} onRateChange={noOp} onResume={noOp} onSelect={noOp} onStop={noOp} onTranslate={noOp} onVoiceChange={noOp} playback="idle" rate={100} selected={articles[0]} translating={false} voice="female" /></Screen> };

export const Study: Story = { render: () => <Screen screen="study"><StudyView complete={false} onAnswer={async () => true} onExit={noOp} onNext={noOp} onPracticeMistakes={noOp} onSpeak={noOp} question={question} result={null} summary={{ answered: 0, correct: 0, wrong: 0, newWords: 0 }} title="Everyday English" /></Screen> };

export const AnswerResult: Story = { render: () => <Screen screen="study"><StudyView complete={false} onAnswer={async () => true} onExit={noOp} onNext={() => { document.body.dataset.studyAdvanced = "true"; }} onPracticeMistakes={noOp} onSpeak={noOp} question={question} result={answer} summary={{ answered: 1, correct: 0, wrong: 1, newWords: 1 }} title="Everyday English" /></Screen> };

export const SessionSetup: Story = { render: () => <Screen screen="session"><SessionSetupView home={home} onCancel={noOp} onStart={noOp} /></Screen> };

export const Mistakes: Story = { render: () => <Screen screen="mistakes"><MistakesView datasetName="Everyday English" lastWrongOnly={false} minimum={1} onFilter={noOp} onPractice={noOp} words={words} /></Screen> };

export const Settings: Story = { render: () => <Screen screen="settings"><SettingsView onError={noOp} /></Screen> };
