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
}

export function useStudyFlow(options: StudyFlowOptions) {
  const [session, setSession] = useState<CollectionSession | null>(null);
  const [question, setQuestion] = useState<QuizQuestion | null>(null);
  const [result, setResult] = useState<AnswerResult | null>(null);
  const [complete, setComplete] = useState(false);
  const [questionStarted, setQuestionStarted] = useState(0);

  const begin = useCallback(async (spec: CollectionSpec) => {
    try {
      const nextSession = await startCollection(spec);
      if (nextSession.totalCount === 0) {
        options.onError(options.collectionEmptyMessage);
        return;
      }
      setSession(nextSession);
      setQuestion(await nextQuestion(nextSession.collectionId));
      setQuestionStarted(Date.now());
      setResult(null);
      setComplete(false);
      options.onStart();
    } catch (error) {
      options.onError(error);
    }
  }, [options]);

  const answer = useCallback(async (optionId: string) => {
    if (!session || !question || result) return;
    try {
      const answerResult = await submitAnswer(
        session.collectionId,
        question.questionId,
        optionId,
        Date.now() - questionStarted,
      );
      setResult(answerResult);
      void speak(answerResult.lemma).catch(() => undefined);
    } catch (error) {
      options.onError(error);
    }
  }, [options, question, questionStarted, result, session]);

  const advance = useCallback(async () => {
    if (!session) return;
    try {
      const upcoming = await nextQuestion(session.collectionId);
      setResult(null);
      setQuestion(upcoming);
      setQuestionStarted(Date.now());
      setComplete(upcoming === null);
    } catch (error) {
      options.onError(error);
    }
  }, [options, session]);

  const exit = useCallback(async () => {
    if (session) await finishSession(session.sessionId).catch(() => undefined);
    setSession(null);
    setQuestion(null);
    setResult(null);
    await options.onExit();
  }, [options, session]);

  return { advance, answer, begin, complete, exit, question, result };
}
