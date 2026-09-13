import type { Meta, StoryObj } from "@storybook/react-vite";

import { DatasetsView } from "../features/datasets/DatasetsView";
import { HomeView } from "../features/home/HomeView";
import { MistakesView } from "../features/mistakes/MistakesView";
import { SettingsView } from "../features/settings/SettingsView";
import { StudyView } from "../features/study/StudyView";
import { AppShell, type NavScreen } from "../layout/AppShell";
import type { AnswerResult as AnswerResultDto, DatasetSummary, HomeDto, QuizQuestion, WrongWord } from "../lib/commands";

const datasets: DatasetSummary[] = [
  { id: "core", name: "Everyday English", createdAt: 1, preloaded: true, wordCount: 640 },
  { id: "travel", name: "Travel notes", createdAt: 2, preloaded: false, wordCount: 84 },
  { id: "work", name: "Product vocabulary", createdAt: 3, preloaded: false, wordCount: 126 },
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

function Screen(props: { children: React.ReactNode; screen: NavScreen | "study" }) {
  return <AppShell appName="Polarbear Vocab" footer="Polarbear Vocab · 0.10" onNavigate={() => undefined} screen={props.screen}>{props.children}</AppShell>;
}

const noOp = () => undefined;
const meta = {
  title: "Screens",
  parameters: { controls: { disable: true } },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const Home: Story = { render: () => <Screen screen="home"><HomeView datasets={datasets} home={home} onContinue={noOp} onDatasetChange={noOp} onMistakes={noOp} onPracticeCollection={noOp} /></Screen> };

export const DatasetList: Story = { render: () => <Screen screen="datasets"><DatasetsView datasets={datasets} onChanged={async () => undefined} onError={noOp} onSelect={noOp} onStart={noOp} selectedDatasetId="" /></Screen> };

export const DatasetDetail: Story = { render: () => <Screen screen="datasets"><DatasetsView datasets={datasets} onChanged={async () => undefined} onError={noOp} onSelect={noOp} onStart={noOp} selectedDatasetId="core" /></Screen> };

export const Study: Story = { render: () => <Screen screen="study"><StudyView complete={false} onAnswer={async () => true} onExit={noOp} onNext={noOp} onSpeak={noOp} question={question} result={null} title="Everyday English" /></Screen> };

export const AnswerResult: Story = { render: () => <Screen screen="study"><StudyView complete={false} onAnswer={async () => true} onExit={noOp} onNext={noOp} onSpeak={noOp} question={question} result={answer} title="Everyday English" /></Screen> };

export const Mistakes: Story = { render: () => <Screen screen="mistakes"><MistakesView datasetName="Everyday English" lastWrongOnly={false} minimum={1} onFilter={noOp} onPractice={noOp} words={words} /></Screen> };

export const Settings: Story = { render: () => <Screen screen="settings"><SettingsView onError={noOp} /></Screen> };
