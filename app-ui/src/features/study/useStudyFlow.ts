import { useCallback, useEffect, useState } from "react";

import {
  finishSession,
  getResumableSession,
  nextQuestion,
  speak,
  startCollection,
  submitAnswer,
  type AnswerResult,
  type CollectionSession,
  type CollectionSpec,
  type QuizQuestion,
  type SpeechVoice,
} from "../../lib/commands";

interface StudyFlowOptions {
  collectionEmptyMessage: string;
  onError: (error: unknown) => void;
  onExit: () => Promise<void>;
  onStart: () => void;
  speechLocale: string;
  speechRate: number;
  speechVoice: SpeechVoice;
}

export interface SessionSummary {
  answered: number;
  correct: number;
  wrong: number;
  newWords: number;
}

export function useStudyFlow(options: StudyFlowOptions) {
  const { collectionEmptyMessage, onError, onExit, onStart, speechLocale, speechRate, speechVoice } = options;
  const [session, setSession] = useState<CollectionSession | null>(null);
  const [question, setQuestion] = useState<QuizQuestion | null>(null);
  const [result, setResult] = useState<AnswerResult | null>(null);
  const [complete, setComplete] = useState(false);
  const [questionStarted, setQuestionStarted] = useState(0);
  const [summary, setSummary] = useState<SessionSummary>({ answered: 0, correct: 0, wrong: 0, newWords: 0 });
  const [wrongSenseUids, setWrongSenseUids] = useState<string[]>([]);
  const [resumableSession, setResumableSession] = useState<CollectionSession | null>(null);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void getResumableSession().then(setResumableSession).catch(onError);
  }, [onError]);

  const begin = useCallback(async (spec: CollectionSpec, limit?: number) => {
    try {
      const nextSession = await startCollection(spec, limit);
      if (nextSession.totalCount === 0) {
        onError(collectionEmptyMessage);
        return;
      }
      const firstQuestion = await nextQuestion(nextSession.collectionId);
      if (!firstQuestion) {
        await finishSession(nextSession.sessionId).catch(() => undefined);
        onError(collectionEmptyMessage);
        return;
      }
      setSession(nextSession);
      setQuestion(firstQuestion);
      setQuestionStarted(Date.now());
      setResult(null);
      setComplete(false);
      setSummary({ answered: 0, correct: 0, wrong: 0, newWords: 0 });
      setWrongSenseUids([]);
      setResumableSession(null);
      onStart();
    } catch (error) {
      onError(error);
    }
  }, [collectionEmptyMessage, onError, onStart]);

  const resume = useCallback(async () => {
    if (!resumableSession) return;
    try {
      const pending = await nextQuestion(resumableSession.collectionId);
      if (!pending) return;
      setSession(resumableSession);
      setQuestion(pending);
      setQuestionStarted(Date.now());
      setResult(null);
      setComplete(false);
      setSummary({
        answered: resumableSession.answeredCount,
        correct: resumableSession.correctCount,
        wrong: resumableSession.wrongCount,
        newWords: resumableSession.newWordCount,
      });
      setWrongSenseUids(resumableSession.wrongSenseUids);
      setResumableSession(null);
      onStart();
    } catch (error) {
      onError(error);
    }
  }, [onError, onStart, resumableSession]);

  const answer = useCallback(async (optionId: string) => {
    if (!session || !question || result) return false;
    try {
      const answerResult = await submitAnswer(
        session.collectionId,
        question.questionId,
        optionId,
        Date.now() - questionStarted,
      );
      setResult(answerResult);
      setSummary((current) => ({
        answered: current.answered + 1,
        correct: current.correct + Number(answerResult.correct),
        wrong: current.wrong + Number(!answerResult.correct),
        newWords: current.newWords + Number(answerResult.wasNew),
      }));
      if (!answerResult.correct) {
        setWrongSenseUids((current) => current.includes(answerResult.correctSenseUid)
          ? current
          : [...current, answerResult.correctSenseUid]);
      }
      void speak(answerResult.lemma, speechLocale, speechRate, speechVoice).catch(() => undefined);
      return true;
    } catch (error) {
      onError(error);
      return false;
    }
  }, [onError, question, questionStarted, result, session, speechLocale, speechRate, speechVoice]);

  const advance = useCallback(async () => {
    if (!session) return;
    try {
      const upcoming = await nextQuestion(session.collectionId);
      setResult(null);
      setQuestion(upcoming);
      setQuestionStarted(Date.now());
      setComplete(upcoming === null);
    } catch (error) {
      onError(error);
    }
  }, [onError, session]);

  const exit = useCallback(async () => {
    if (session) await finishSession(session.sessionId).catch(() => undefined);
    setSession(null);
    setQuestion(null);
    setResult(null);
    setResumableSession(null);
    await onExit();
  }, [onExit, session]);

  const practiceMistakes = useCallback(async () => {
    if (wrongSenseUids.length === 0) return;
    if (session) await finishSession(session.sessionId).catch(() => undefined);
    await begin({ type: "custom", senseUids: wrongSenseUids });
  }, [begin, session, wrongSenseUids]);

  return { advance, answer, begin, complete, exit, practiceMistakes, question, result, resumableSession, resume, summary };
}
