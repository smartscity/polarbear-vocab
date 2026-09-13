import type { AnswerResult as AnswerResultDto, QuestionOption } from "../../lib/commands";
import type { ReactNode } from "react";
import { Button } from "../primitives/Button";

export function QuestionCard(props: { count: string; prompt: string; children: ReactNode }) {
  return <section className="question-card"><p className="question-count">{props.count}</p><h1 className="question-prompt" lang="zh-CN">{props.prompt}</h1>{props.children}</section>;
}

export function AnswerOption(props: {
  onSelect: (optionId: string) => void;
  option: QuestionOption;
  selected?: boolean;
  shortcut: string;
}) {
  return (
    <button className="answer-option" data-state={props.selected ? "selected" : "idle"} onClick={() => props.onSelect(props.option.optionId)} type="button">
      <kbd>{props.shortcut}</kbd><span>{props.option.lemma}</span>
    </button>
  );
}

interface AnswerResultProps {
  nextLabel: string;
  onNext: () => void;
  onSpeak: (text: string) => void;
  result: AnswerResultDto;
  spaceLabel: string;
}

export function AnswerResult({ nextLabel, onNext, onSpeak, result, spaceLabel }: AnswerResultProps) {
  return (
    <section className="question-card answer-result">
      <p className="result-mark" data-correct={result.correct}>{result.correct ? "✓" : "×"}</p>
      <button className="speak-title" onClick={() => onSpeak(result.lemma)} type="button"><h2 className="pb-display">{result.lemma}</h2><span aria-hidden="true">🔊</span></button>
      <p className="ipa">{result.ipa}</p><p className="gloss" lang="zh-CN">{result.zhGloss}</p>
      <button className="example" onClick={() => onSpeak(result.exampleEn)} type="button"><span>{result.exampleEn}<small lang="zh-CN">{result.exampleZh}</small></span><span aria-hidden="true">🔊</span></button>
      <Button onClick={onNext} variant="primary">{nextLabel} <kbd>{spaceLabel}</kbd></Button>
    </section>
  );
}
