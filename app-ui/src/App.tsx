import { useCallback, useMemo, useState } from "react";

import { DatasetsView } from "./features/datasets/DatasetsView";
import { HomeView } from "./features/home/HomeView";
import { LexiconView } from "./features/lexicon/LexiconView";
import { useLexiconSearch } from "./features/lexicon/useLexiconSearch";
import { ListeningView } from "./features/listening/ListeningView";
import { useListeningFlow } from "./features/listening/useListeningFlow";
import { MistakesView } from "./features/mistakes/MistakesView";
import { useMistakeFlow } from "./features/mistakes/useMistakeFlow";
import { SettingsView } from "./features/settings/SettingsView";
import { SessionSetupView } from "./features/study/SessionSetupView";
import { StudyView } from "./features/study/StudyView";
import { useStudyFlow } from "./features/study/useStudyFlow";
import { AppShell, type NavScreen } from "./layout/AppShell";
import { speak, type CollectionSpec, type DatasetSummary, type HomeDto } from "./lib/commands";
import { useAppData } from "./lib/useAppData";
import { useI18n } from "./lib/i18n";

type Screen = NavScreen | "session" | "study";
type SessionCollection = "unseen" | "mistakes" | "all";

export function App() {
  const { speechLocale, speechRatePercent, speechVoice, t } = useI18n();
  const data = useAppData();
  const [screen, setScreen] = useState<Screen>("home");
  const [sessionCollection, setSessionCollection] = useState<SessionCollection>("unseen");
  const showMistakesScreen = useCallback(() => setScreen("mistakes"), []);
  const showStudyScreen = useCallback(() => setScreen("study"), []);
  const mistakes = useMistakeFlow(data.selectedDatasetId, data.reportError, showMistakesScreen);
  const listening = useListeningFlow(data.reportError);
  const lexicon = useLexiconSearch(data.reportError);
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
    speechVoice,
  });
  const currentDataset = useMemo(
    () => data.datasets.find((dataset) => dataset.id === data.selectedDatasetId),
    [data.datasets, data.selectedDatasetId],
  );
  const openSession = useCallback(async (datasetId: string, collection: SessionCollection = "unseen") => {
    if (datasetId !== data.selectedDatasetId) await data.changeDataset(datasetId);
    setSessionCollection(collection);
    setScreen("session");
  }, [data.changeDataset, data.selectedDatasetId]);
  const navigate = (destination: NavScreen) => {
    if (destination === "mistakes") void mistakes.show(1, false);
    else setScreen(destination);
  };

  return (
    <AppShell appName={data.appInfo.name} footer="Copyright © 2020-2026 smartscity All rights reserved." onNavigate={navigate} screen={screen}>
      {data.error ? <ErrorBanner message={data.error} onClose={() => data.setError(null)} /> : null}
      <ScreenView
        currentDataset={currentDataset}
        data={data}
        mistakes={mistakes}
        listening={listening}
        lexicon={lexicon}
        screen={screen}
        sessionCollection={sessionCollection}
        study={study}
        onOpenSession={openSession}
        onCancelSession={() => setScreen("home")}
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
type ListeningFlow = ReturnType<typeof useListeningFlow>;
type LexiconSearch = ReturnType<typeof useLexiconSearch>;

interface ScreenViewProps {
  currentDataset?: DatasetSummary;
  data: AppData;
  mistakes: MistakeFlow;
  listening: ListeningFlow;
  lexicon: LexiconSearch;
  screen: Screen;
  sessionCollection: SessionCollection;
  study: StudyFlow;
  onOpenSession: (datasetId: string, collection?: SessionCollection) => Promise<void>;
  onCancelSession: () => void;
}

function ScreenView(props: ScreenViewProps) {
  const { speechLocale, speechRatePercent, speechVoice, t } = useI18n();
  if (props.screen === "home") {
    return <HomeScreen data={props.data} home={props.data.home} mistakes={props.mistakes} onOpenSession={props.onOpenSession} study={props.study} />;
  }
  if (props.screen === "datasets") {
    return (
      <DatasetsView
        datasets={props.data.datasets}
        onChanged={props.data.refreshDatasets}
        onError={props.data.reportError}
        onSelect={(datasetId) => void props.data.changeDataset(datasetId)}
        onStart={(datasetId) => void props.onOpenSession(datasetId)}
        selectedDatasetId={props.data.selectedDatasetId}
      />
    );
  }
  if (props.screen === "lexicon") {
    return (
      <LexiconView
        busy={props.lexicon.busy}
        onPractice={(senseUid) => void props.study.begin({ type: "custom", senseUids: [senseUid] })}
        onQueryChange={props.lexicon.setQuery}
        onSearch={() => void props.lexicon.search()}
        onSpeak={(text) => void speak(text, speechLocale, speechRatePercent / 100, speechVoice).catch(props.data.reportError)}
        onToggleVocabulary={(entry) => void props.lexicon.toggleVocabulary(entry)}
        query={props.lexicon.query}
        results={props.lexicon.results}
        searched={props.lexicon.searched}
      />
    );
  }
  if (props.screen === "settings") return <SettingsView onError={props.data.reportError} />;
  if (props.screen === "session" && props.data.home) {
    return (
      <SessionSetupView
        home={props.data.home}
        initialCollection={props.sessionCollection}
        onCancel={props.onCancelSession}
        onStart={(spec, limit) => void props.study.begin(spec, limit)}
      />
    );
  }
  if (props.screen === "listening") {
    return (
      <ListeningView
        articles={props.listening.articles}
        onDelete={() => void props.listening.remove()}
        onImport={() => void props.listening.chooseFile()}
        onError={props.data.reportError}
        onPause={() => void props.listening.pause()}
        onPlay={() => void props.listening.play()}
        onRateChange={(rate) => void props.listening.changeRate(rate)}
        onResume={() => void props.listening.resume()}
        onSelect={(articleId) => void props.listening.select(articleId)}
        onStop={() => void props.listening.stop()}
        onVoiceChange={(voice) => void props.listening.changeVoice(voice)}
        playback={props.listening.playback}
        rate={props.listening.speechRatePercent}
        selected={props.listening.selected}
        voice={props.listening.speechVoice}
      />
    );
  }
  if (props.screen === "study") {
    return (
      <StudyView
        complete={props.study.complete}
        onAnswer={props.study.answer}
        onExit={() => void props.study.exit()}
        onNext={() => void props.study.advance()}
        onPracticeMistakes={() => void props.study.practiceMistakes()}
        onSpeak={(text) => void speak(text, speechLocale, speechRatePercent / 100, speechVoice)}
        question={props.study.question}
        result={props.study.result}
        summary={props.study.summary}
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

function HomeScreen(props: {
  data: AppData;
  home: HomeDto | null;
  mistakes: MistakeFlow;
  onOpenSession: ScreenViewProps["onOpenSession"];
  study: StudyFlow;
}) {
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
      onContinue={() => void props.onOpenSession(datasetId, "unseen")}
      onDatasetChange={(id) => void props.data.changeDataset(id)}
      onMistakes={(minimum, lastWrongOnly) => void props.mistakes.show(minimum, lastWrongOnly)}
      onPracticeCollection={(type) => void props.study.begin({ type, datasetId })}
      onResume={() => void props.study.resume()}
      resumableCount={props.study.resumableSession ? props.study.resumableSession.totalCount - props.study.resumableSession.answeredCount : undefined}
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
