import { EmptyState } from "../../design-system/components/EmptyState";
import { MistakeFilter } from "../../design-system/components/MistakeFilter";
import { PageHeader } from "../../design-system/components/PageHeader";
import { Button } from "../../design-system/primitives/Button";
import type { WrongWord } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface MistakesViewProps {
  datasetName: string;
  words: WrongWord[];
  minimum: number;
  lastWrongOnly: boolean;
  onPractice: () => void;
  onFilter: (minimum: number, lastWrongOnly: boolean) => void;
}

export function MistakesView(props: MistakesViewProps) {
  const { t } = useI18n();
  const options = [
    { label: t("mistakes.all"), minimum: 1, lastWrongOnly: false },
    { label: "≥ 2", minimum: 2, lastWrongOnly: false },
    { label: "≥ 3", minimum: 3, lastWrongOnly: false },
    { label: "≥ 5", minimum: 5, lastWrongOnly: false },
    { label: t("mistakes.lastWrong"), minimum: 1, lastWrongOnly: true },
  ];
  return (
    <section className="mistakes-page">
      <PageHeader eyebrow={props.datasetName} title={t("nav.mistakes")} />
      <MistakeFilter lastWrongOnly={props.lastWrongOnly} minimum={props.minimum} onChange={props.onFilter} options={options} />
      {props.words.length === 0 ? <EmptyState>{t("mistakes.empty")}</EmptyState> : <WordList words={props.words} />}
      <Button className="practice-button" disabled={props.words.length === 0} onClick={props.onPractice} variant="primary">{t("mistakes.practice")}</Button>
    </section>
  );
}

function WordList({ words }: { words: WrongWord[] }) {
  const { t } = useI18n();
  return (
    <div className="word-list">
      {words.map((word) => (
        <article key={word.senseUid}><div><strong>{word.lemma}</strong><span>{word.ipa}</span><small lang="zh-CN">{word.zhGloss}</small></div><p><span>{t("mistakes.countWrong", { count: word.wrongCount })}</span><span>{t("mistakes.countCorrect", { count: word.correctCount })}</span></p></article>
      ))}
    </div>
  );
}
