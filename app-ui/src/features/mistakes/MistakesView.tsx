import type { WrongWord } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface MistakesViewProps {
  datasetName: string;
  words: WrongWord[];
  minimum: number;
  lastWrongOnly: boolean;
  onBack: () => void;
  onPractice: () => void;
  onFilter: (minimum: number, lastWrongOnly: boolean) => void;
}

export function MistakesView(props: MistakesViewProps) {
  const { t } = useI18n();
  const filters = [[t("mistakes.all"), 1, false], ["≥ 2", 2, false], ["≥ 3", 3, false], ["≥ 5", 5, false], [t("mistakes.lastWrong"), 1, true]] as const;
  return (
    <section className="mistakes-page">
      <div className="study-toolbar"><button type="button" onClick={props.onBack}>← {t("nav.home")}</button><span>{t("nav.mistakes")} · {props.datasetName}</span></div>
      <div className="filter-row">
        {filters.map(([label, minimum, lastWrongOnly]) => (
          <button className={props.minimum === minimum && props.lastWrongOnly === lastWrongOnly ? "active" : ""} key={label} type="button" onClick={() => props.onFilter(minimum, lastWrongOnly)}>{label}</button>
        ))}
      </div>
      <div className="word-list">
        {props.words.length === 0 ? <p className="empty-state">{t("mistakes.empty")}</p> : null}
        {props.words.map((word) => (
          <article key={word.senseUid}><div><strong>{word.lemma}</strong><span>{word.ipa}</span><small lang="zh-CN">{word.zhGloss}</small></div><p><span>{t("mistakes.countWrong", { count: word.wrongCount })}</span><span>{t("mistakes.countCorrect", { count: word.correctCount })}</span></p></article>
        ))}
      </div>
      <button className="primary-button practice-button" type="button" disabled={props.words.length === 0} onClick={props.onPractice}>{t("mistakes.practice")}</button>
    </section>
  );
}
