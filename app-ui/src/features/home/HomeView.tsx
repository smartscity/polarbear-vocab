import type { DatasetSummary, HomeDto } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface HomeViewProps {
  datasets: DatasetSummary[];
  home: HomeDto;
  onDatasetChange: (id: string) => void;
  onContinue: () => void;
  onMistakes: (minimum: number, lastWrongOnly: boolean) => void;
  onPracticeCollection: (type: "answered" | "correct") => void;
}

export function HomeView(props: HomeViewProps) {
  const { datasets, home, onDatasetChange, onContinue, onMistakes, onPracticeCollection } = props;
  const { t } = useI18n();
  const maxActivity = Math.max(1, ...home.dailyActivity.map((day) => day.attemptCount));
  const buckets = [
    [t("mistakes.wrongAtLeast", { count: 1 }), home.mistakeBuckets.atLeastOne, 1, false],
    [t("mistakes.wrongAtLeast", { count: 2 }), home.mistakeBuckets.atLeastTwo, 2, false],
    [t("mistakes.wrongAtLeast", { count: 3 }), home.mistakeBuckets.atLeastThree, 3, false],
    [t("mistakes.wrongAtLeast", { count: 5 }), home.mistakeBuckets.atLeastFive, 5, false],
    [t("mistakes.lastWrong"), home.mistakeBuckets.lastWrong, 1, true],
  ] as const;

  return (
    <>
      <section className="home-hero">
        <label className="dataset-picker">
          <span>{t("home.dataset")}</span>
          <select value={home.dataset.id} onChange={(event) => onDatasetChange(event.target.value)}>
            {datasets.map((dataset) => (
              <option key={dataset.id} value={dataset.id}>
                {dataset.name}
              </option>
            ))}
          </select>
        </label>
        <div className="progress-copy">
          {t("home.explored", {
            answered: home.progress.answered.toLocaleString(),
            total: home.progress.total.toLocaleString(),
          })}
        </div>
        <div className="progress-track" aria-hidden="true">
          <span style={{ width: `${home.progress.total === 0 ? 0 : (home.progress.answered / home.progress.total) * 100}%` }} />
        </div>
        <button className="primary-button" type="button" onClick={onContinue} disabled={home.progress.unseen === 0}>
          {home.progress.unseen > 0 ? t("home.continue") : t("home.allExplored")}
        </button>
      </section>

      <section className="panel activity-panel">
        <div className="section-heading">
          <div><p className="eyebrow">{t("home.last30Days")}</p><h2>{t("home.activity")}</h2></div>
          <span>{t("home.historyOnly")}</span>
        </div>
        <div className="activity-chart" aria-label={t("home.activityAria")}>
          {home.dailyActivity.map((day) => (
            <div className="activity-column" key={day.localDate} title={`${day.localDate}: ${day.attemptCount}`}>
              <span style={{ height: `${(day.attemptCount / maxActivity) * 100}%` }} />
            </div>
          ))}
        </div>
        <div className="stat-grid">
          <Stat label={t("stats.explored")} value={home.totals.explored} onClick={() => onPracticeCollection("answered")} />
          <Stat label={t("stats.correct")} value={home.totals.correct} onClick={() => onPracticeCollection("correct")} />
          <Stat label={t("stats.wrong")} value={home.totals.mistakes} onClick={() => onMistakes(1, false)} />
        </div>
      </section>

      <section className="panel">
        <div className="section-heading"><div><p className="eyebrow">{t("home.historyCollections")}</p><h2>{t("nav.mistakes")}</h2></div></div>
        <div className="bucket-grid">
          {buckets.map(([label, value, minimum, lastWrongOnly]) => (
            <button key={label} type="button" disabled={value === 0} onClick={() => onMistakes(minimum, lastWrongOnly)}>
              <span>{label}</span><strong>{value}</strong>
            </button>
          ))}
        </div>
      </section>
    </>
  );
}

function Stat({ label, value, onClick }: { label: string; value: number; onClick: () => void }) {
  return <button disabled={value === 0} onClick={onClick} type="button"><span>{label}</span><strong>{value.toLocaleString()}</strong></button>;
}
