import { ActivityChart } from "../../design-system/components/ActivityChart";
import { DatasetSwitcher } from "../../design-system/components/DatasetSwitcher";
import { MetricRow } from "../../design-system/components/MetricRow";
import { Button } from "../../design-system/primitives/Button";
import type { DatasetSummary, HomeDto } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

const logo = new URL("../../../../src-tauri/icons/128x128@2x.png", import.meta.url).href;

interface HomeViewProps {
  datasets: DatasetSummary[];
  home: HomeDto;
  onDatasetChange: (id: string) => void;
  onContinue: () => void;
  onMistakes: (minimum: number, lastWrongOnly: boolean) => void;
  onPracticeCollection: (type: "answered" | "correct") => void;
  onResume: () => void;
  resumableCount?: number;
}

export function HomeView(props: HomeViewProps) {
  const { t } = useI18n();
  const buckets = [
    [t("mistakes.wrongAtLeast", { count: 1 }), props.home.mistakeBuckets.atLeastOne, 1, false],
    [t("mistakes.wrongAtLeast", { count: 2 }), props.home.mistakeBuckets.atLeastTwo, 2, false],
    [t("mistakes.wrongAtLeast", { count: 3 }), props.home.mistakeBuckets.atLeastThree, 3, false],
    [t("mistakes.wrongAtLeast", { count: 5 }), props.home.mistakeBuckets.atLeastFive, 5, false],
    [t("mistakes.lastWrong"), props.home.mistakeBuckets.lastWrong, 1, true],
  ] as const;
  const metrics = [
    { label: t("stats.explored"), value: props.home.totals.explored, onClick: () => props.onPracticeCollection("answered") },
    { label: t("stats.correct"), value: props.home.totals.correct, onClick: () => props.onPracticeCollection("correct") },
    { label: t("stats.wrong"), value: props.home.totals.mistakes, onClick: () => props.onMistakes(1, false) },
  ];
  return (
    <div className="home-page">
      <section className="home-hero">
        <div className="home-hero-heading">
          <DatasetSwitcher datasets={props.datasets} label={t("home.dataset")} onChange={props.onDatasetChange} value={props.home.dataset.id} />
          <img alt="Polarbear Vocab" className="home-hero-logo" height="76" src={logo} width="76" />
        </div>
        <div className="progress-copy">{t("home.explored", { answered: props.home.progress.answered.toLocaleString(), total: props.home.progress.total.toLocaleString() })}</div>
        <div aria-hidden="true" className="progress-track"><span style={{ width: `${progressPercent(props.home)}%` }} /></div>
        <Button disabled={props.home.progress.unseen === 0} onClick={props.onContinue} variant="primary">
          {props.home.progress.unseen > 0 ? t("home.continue") : t("home.allExplored")}
        </Button>
        {props.resumableCount ? <Button onClick={props.onResume}>{t("home.resumeSession", { count: props.resumableCount })}</Button> : null}
      </section>
      <section className="home-section">
        <div className="section-heading"><div><p className="pb-eyebrow">{t("home.last30Days")}</p><h2>{t("home.activity")}</h2></div><span>{t("home.historyOnly")}</span></div>
        <ActivityChart activity={props.home.dailyActivity} ariaLabel={t("home.activityAria")} />
        <MetricRow metrics={metrics} />
      </section>
      <section className="home-section">
        <div className="section-heading"><div><p className="pb-eyebrow">{t("home.historyCollections")}</p><h2>{t("nav.mistakes")}</h2></div></div>
        <div className="bucket-grid">
          {buckets.map(([label, value, minimum, lastWrongOnly]) => (
            <button disabled={value === 0} key={label} onClick={() => props.onMistakes(minimum, lastWrongOnly)} type="button"><span>{label}</span><strong>{value}</strong></button>
          ))}
        </div>
      </section>
    </div>
  );
}

function progressPercent(home: HomeDto): number {
  return home.progress.total === 0 ? 0 : (home.progress.answered / home.progress.total) * 100;
}
