import { useCallback, useMemo, useState } from "react";

import { DatasetsView } from "./features/datasets/DatasetsView";
import { HomeView } from "./features/home/HomeView";
import { MistakesView } from "./features/mistakes/MistakesView";
import { useMistakeFlow } from "./features/mistakes/useMistakeFlow";
import { SettingsView } from "./features/settings/SettingsView";
import { StudyView } from "./features/study/StudyView";
import { useStudyFlow } from "./features/study/useStudyFlow";
import { AppShell, type NavScreen } from "./layout/AppShell";
import { speak, type CollectionSpec, type DatasetSummary, type HomeDto } from "./lib/commands";
import { useAppData } from "./lib/useAppData";
import { useI18n } from "./lib/i18n";

type Screen = NavScreen | "study";

export function App() {
  const { speechLocale, speechRatePercent, t } = useI18n();
  const data = useAppData();
  const [screen, setScreen] = useState<Screen>("home");
  const showMistakesScreen = useCallback(() => setScreen("mistakes"), []);
  const showStudyScreen = useCallback(() => setScreen("study"), []);
  const mistakes = useMistakeFlow(data.selectedDatasetId, data.reportError, showMistakesScreen);
  const exitStudy = useCallback(async () => {
    setScreen("home");
    if (data.selectedDatasetId) {
      await data.refreshHome(data.selectedDatasetId).catch(data.reportError);
    }
  }, [data.refreshHome, data.reportError, data.selectedDatasetId]);
  const study = useStudyFlow({
    collectionEmptyMessage: t("error.collectionEmpty"),
    onError: data.reportError,
    onExit: exitStudy,
    onStart: showStudyScreen,
    speechLocale,
    speechRate: speechRatePercent / 100,
  });
  const currentDataset = useMemo(
    () => data.datasets.find((dataset) => dataset.id === data.selectedDatasetId),
    [data.datasets, data.selectedDatasetId],
  );
  const navigate = (destination: NavScreen) => {
    if (destination === "mistakes") void mistakes.show(1, false);
    else setScreen(destination);
  };

  return (
    <AppShell appName={data.appInfo.name} footer={`${data.appInfo.name} · ${data.appInfo.version}`} onNavigate={navigate} screen={screen}>
      {data.error ? <ErrorBanner message={data.error} onClose={() => data.setError(null)} /> : null}
      <ScreenView
        currentDataset={currentDataset}
        data={data}
        mistakes={mistakes}
        screen={screen}
        study={study}
      />
    </AppShell>
  );
}

function ErrorBanner({ message, onClose }: { message: string; onClose: () => void }) {
  const { t } = useI18n();
  return <div className="pb-error-banner" role="alert">{message}<button aria-label={t("common.close")} type="button" onClick={onClose}>×</button></div>;
}

type AppData = ReturnType<typeof useAppData>;
type StudyFlow = ReturnType<typeof useStudyFlow>;
type MistakeFlow = ReturnType<typeof useMistakeFlow>;

interface ScreenViewProps {
  currentDataset?: DatasetSummary;
  data: AppData;
  mistakes: MistakeFlow;
  screen: Screen;
  study: StudyFlow;
}

function ScreenView(props: ScreenViewProps) {
  const { speechLocale, speechRatePercent, t } = useI18n();
  if (props.screen === "home") {
    return <HomeScreen data={props.data} home={props.data.home} mistakes={props.mistakes} study={props.study} />;
  }
  if (props.screen === "datasets") {
    return (
      <DatasetsView
        datasets={props.data.datasets}
        onChanged={props.data.refreshDatasets}
        onError={props.data.reportError}
        onSelect={(datasetId) => void props.data.changeDataset(datasetId)}
        onStart={(datasetId) => void props.study.begin({ type: "dataset", datasetId })}
        selectedDatasetId={props.data.selectedDatasetId}
      />
    );
  }
  if (props.screen === "settings") return <SettingsView onError={props.data.reportError} />;
  if (props.screen === "study") {
    return (
      <StudyView
        complete={props.study.complete}
        onAnswer={props.study.answer}
        onExit={() => void props.study.exit()}
        onNext={() => void props.study.advance()}
        onSpeak={(text) => void speak(text, speechLocale, speechRatePercent / 100)}
        question={props.study.question}
        result={props.study.result}
        title={props.currentDataset?.name ?? t("home.dataset")}
      />
    );
  }
  return (
    <MistakesView
      datasetName={props.currentDataset?.name ?? t("mistakes.all")}
      lastWrongOnly={props.mistakes.filter.lastWrongOnly}
      minimum={props.mistakes.filter.minimum}
      onFilter={(minimum, lastWrongOnly) => void props.mistakes.show(minimum, lastWrongOnly)}
      onPractice={() => void props.study.begin(mistakeSpec(props.data.selectedDatasetId, props.mistakes.filter))}
      words={props.mistakes.words}
    />
  );
}

function HomeScreen(props: { data: AppData; home: HomeDto | null; mistakes: MistakeFlow; study: StudyFlow }) {
  const { t } = useI18n();
  if (!props.home) {
    const message = "__TAURI_INTERNALS__" in window ? t("error.lexiconLoading") : t("error.desktopOnly");
    return <section className="loading-panel"><h2>{message}</h2></section>;
  }
  const datasetId = props.data.selectedDatasetId;
  return (
    <HomeView
      datasets={props.data.datasets}
      home={props.home}
      onContinue={() => void props.study.begin({ type: "unseen", datasetId })}
      onDatasetChange={(id) => void props.data.changeDataset(id)}
      onMistakes={(minimum, lastWrongOnly) => void props.mistakes.show(minimum, lastWrongOnly)}
      onPracticeCollection={(type) => void props.study.begin({ type, datasetId })}
    />
  );
}

function mistakeSpec(
  datasetId: string,
  filter: { minimum: number; lastWrongOnly: boolean },
): CollectionSpec {
  return filter.lastWrongOnly
    ? { type: "lastWrong", datasetId }
    : { type: "wrong", datasetId, minWrongCount: filter.minimum };
}
