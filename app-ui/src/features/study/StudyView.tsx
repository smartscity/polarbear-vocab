import { useEffect } from "react";

import type { AnswerResult, QuizQuestion } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface StudyViewProps {
  title: string;
  question: QuizQuestion | null;
  result: AnswerResult | null;
  complete: boolean;
  onAnswer: (optionId: string) => void;
  onNext: () => void;
  onExit: () => void;
  onSpeak: (text: string) => void;
}

export function StudyView(props: StudyViewProps) {
  const { question, result, complete, onAnswer, onNext, onExit, onSpeak } = props;
  const { t } = useI18n();
  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onExit();
      if (result && event.code === "Space") onNext();
      if (result && event.key.toLowerCase() === "r") onSpeak(result.lemma);
      if (result && event.key.toLowerCase() === "s") onSpeak(result.exampleEn);
      if (!result && question && ["1", "2", "3", "4"].includes(event.key)) {
        const option = question.options[Number(event.key) - 1];
        if (option) onAnswer(option.optionId);
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onAnswer, onExit, onNext, onSpeak, question, result]);

  if (complete) {
    return (
      <section className="study-card result-card">
        <p className="result-mark correct">✓</p><h2>{t("study.complete")}</h2>
        <p>{t("study.saved")}</p>
        <button className="primary-button" type="button" onClick={onExit}>{t("study.backHome")}</button>
      </section>
    );
  }
  return (
    <>
      <div className="study-toolbar"><button type="button" onClick={onExit}>← {props.title}</button><span>{t("study.answered", { count: question?.answeredCount ?? 0 })}</span></div>
      {result ? <AnswerCard result={result} onNext={onNext} onSpeak={onSpeak} nextLabel={t("study.next")} spaceLabel={t("study.space")} /> : question ? (
        <section className="study-card">
          <p className="question-count">{question.answeredCount + 1} / {question.totalCount}</p>
          <h2 lang="zh-CN">{question.promptZh}</h2>
          <div className="option-list">
            {question.options.map((option, index) => (
              <button key={option.optionId} type="button" onClick={() => onAnswer(option.optionId)}><kbd>{index + 1}</kbd><span>{option.lemma}</span></button>
            ))}
          </div>
        </section>
      ) : <section className="study-card"><p>{t("study.loading")}</p></section>}
    </>
  );
}

function AnswerCard({ result, onNext, onSpeak, nextLabel, spaceLabel }: { result: AnswerResult; onNext: () => void; onSpeak: (text: string) => void; nextLabel: string; spaceLabel: string }) {
  return (
    <section className="study-card result-card">
      <p className={`result-mark ${result.correct ? "correct" : "wrong"}`}>{result.correct ? "✓" : "×"}</p>
      <button className="speak-title" type="button" onClick={() => onSpeak(result.lemma)}><h2>{result.lemma}</h2><span>🔊</span></button>
      <p className="ipa">{result.ipa}</p><p className="gloss" lang="zh-CN">{result.zhGloss}</p>
      <button className="example" type="button" onClick={() => onSpeak(result.exampleEn)}><span>{result.exampleEn}<small lang="zh-CN">{result.exampleZh}</small></span><span>🔊</span></button>
      <button className="primary-button" type="button" onClick={onNext}>{nextLabel} <kbd>{spaceLabel}</kbd></button>
    </section>
  );
}
