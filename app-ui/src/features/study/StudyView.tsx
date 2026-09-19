import { useEffect, useState, type MouseEvent } from "react";

import { AnswerOption, AnswerResult, QuestionCard } from "../../design-system/components/Question";
import { Button } from "../../design-system/primitives/Button";
import type { AnswerResult as AnswerResultDto, QuizQuestion } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import type { SessionSummary } from "./useStudyFlow";

interface StudyViewProps {
  title: string;
  question: QuizQuestion | null;
  result: AnswerResultDto | null;
  complete: boolean;
  onAnswer: (optionId: string) => Promise<boolean>;
  onNext: () => void;
  onExit: () => void;
  onSpeak: (text: string) => void;
  onPracticeMistakes: () => void;
  summary: SessionSummary;
}

export function StudyView(props: StudyViewProps) {
  const { t } = useI18n();
  const [selectedOption, setSelectedOption] = useState<string>();
  const choose = async (optionId: string) => {
    if (props.result || selectedOption) return;
    setSelectedOption(optionId);
    if (!(await props.onAnswer(optionId))) setSelectedOption(undefined);
  };
  useEffect(() => setSelectedOption(undefined), [props.question?.questionId]);
  useStudyKeyboard({ onAnswer: choose, onExit: props.onExit, onNext: props.onNext, onSpeak: props.onSpeak, question: props.question, result: props.result });
  if (props.complete) {
    return (
      <div className="study-page"><section className="question-card answer-result">
        <p className="result-mark" data-correct="true">✓</p><h1 className="pb-display">{t("study.complete")}</h1>
        <dl className="session-summary">
          <div><dt>{t("study.summaryAnswered")}</dt><dd>{props.summary.answered}</dd></div>
          <div><dt>{t("study.summaryCorrect")}</dt><dd>{props.summary.correct}</dd></div>
          <div><dt>{t("study.summaryWrong")}</dt><dd>{props.summary.wrong}</dd></div>
          <div><dt>{t("study.summaryNew")}</dt><dd>{props.summary.newWords}</dd></div>
        </dl>
        <div className="session-summary-actions">
          <Button disabled={props.summary.wrong === 0} onClick={props.onPracticeMistakes}>{t("study.practiceMistakes")}</Button>
          <Button onClick={props.onExit} variant="primary">{t("study.done")}</Button>
        </div>
      </section></div>
    );
  }
  return (
    <div
      className={`study-page${props.result ? " study-page--answer-result" : ""}`}
      onClick={props.result ? (event) => advanceFromResult(event, props.onNext) : undefined}
    >
      <div className="study-toolbar"><button onClick={props.onExit} type="button">← {props.title}</button><span>{t("study.answered", { count: props.question?.answeredCount ?? 0 })}</span></div>
      {props.result ? <AnswerResult nextLabel={t("study.next")} onNext={props.onNext} onSpeak={props.onSpeak} result={props.result} spaceLabel={t("study.space")} /> : null}
      {!props.result && props.question ? (
        <QuestionCard count={`${props.question.answeredCount + 1} / ${props.question.totalCount}`} prompt={props.question.promptZh}>
          <div className="answer-options">
            {props.question.options.map((option, index) => <AnswerOption key={option.optionId} onSelect={(id) => void choose(id)} option={option} selected={selectedOption === option.optionId} shortcut={String(index + 1)} />)}
          </div>
        </QuestionCard>
      ) : null}
      {!props.result && !props.question ? <section className="question-card"><p>{t("study.loading")}</p></section> : null}
    </div>
  );
}

function advanceFromResult(event: MouseEvent<HTMLDivElement>, onNext: () => void) {
  const target = event.target;
  if (target instanceof Element && target.closest("button, a, input, select, textarea")) return;
  onNext();
}

type StudyKeyboardProps = Pick<StudyViewProps, "onExit" | "onNext" | "onSpeak" | "question" | "result"> & {
  onAnswer: (optionId: string) => Promise<void>;
};

function useStudyKeyboard(props: StudyKeyboardProps) {
  useEffect(() => {
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onExit();
      if (props.result && event.code === "Space") props.onNext();
      if (props.result && event.key.toLowerCase() === "r") props.onSpeak(props.result.lemma);
      if (props.result && event.key.toLowerCase() === "s") props.onSpeak(props.result.exampleEn);
      if (!props.result && props.question && ["1", "2", "3", "4"].includes(event.key)) {
        const option = props.question.options[Number(event.key) - 1];
        if (option) void props.onAnswer(option.optionId);
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [props.onAnswer, props.onExit, props.onNext, props.onSpeak, props.question, props.result]);
}
