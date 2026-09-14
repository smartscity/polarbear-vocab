import { useCallback, useState } from "react";

import {
  finishSession,
  nextQuestion,
  speak,
  startCollection,
  submitAnswer,
  type AnswerResult,
  type CollectionSession,
  type CollectionSpec,
  type QuizQuestion,
} from "../../lib/commands";

interface StudyFlowOptions {
  collectionEmptyMessage: string;
  onError: (error: unknown) => void;
  onExit: () => Promise<void>;
  onStart: () => void;
  speechLocale: string;
  speechRate: number;
  speechVoice: "male" | "female" | "indian" | "japanese";
}

export function useStudyFlow(options: StudyFlowOptions) {
  const { collectionEmptyMessage, onError, onExit, onStart, speechLocale, speechRate, speechVoice } = options;
  const [session, setSession] = useState<CollectionSession | null>(null);
  const [question, setQuestion] = useState<QuizQuestion | null>(null);
  const [result, setResult] = useState<AnswerResult | null>(null);
  const [complete, setComplete] = useState(false);
  const [questionStarted, setQuestionStarted] = useState(0);

  const begin = useCallback(async (spec: CollectionSpec) => {
    try {
      const nextSession = await startCollection(spec);
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
      onStart();
    } catch (error) {
      onError(error);
    }
  }, [collectionEmptyMessage, onError, onStart]);

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
    await onExit();
  }, [onExit, session]);

  return { advance, answer, begin, complete, exit, question, result };
}
