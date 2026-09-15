import { useEffect, useState } from "react";

import { EmptyState } from "../../design-system/components/EmptyState";
import { PageHeader } from "../../design-system/components/PageHeader";
import { Button } from "../../design-system/primitives/Button";
import { ConfirmDialog } from "../../design-system/primitives/Dialogs";
import { SelectControl } from "../../design-system/primitives/SelectControl";
import {
  addToMyVocabulary,
  searchLexicon,
  SPEECH_VOICES,
  type Article,
  type LexiconEntry,
  type SpeechVoice,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import type { PlaybackState } from "./useListeningFlow";
import { lexiconLookupCandidates } from "./wordLookup";

interface ListeningViewProps {
  articles: Article[];
  onDelete: () => void;
  onImport: () => void;
  onError: (error: unknown) => void;
  onPause: () => void;
  onPlay: () => void;
  onResume: () => void;
  onSelect: (articleId: string) => void;
  onStop: () => void;
  onRateChange: (rate: number) => void;
  onVoiceChange: (voice: SpeechVoice) => void;
  playback: PlaybackState;
  rate: number;
  selected?: Article;
  voice: SpeechVoice;
}

export function ListeningView(props: ListeningViewProps) {
  const { t } = useI18n();
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [selectedWord, setSelectedWord] = useState<LexiconEntry | null>(null);
  const [wordBusy, setWordBusy] = useState(false);
  const voiceOptions = SPEECH_VOICES.map((value) => ({
    label: t(`settings.voice.${value}`),
    value,
  }));
  const rateOptions = [50, 100, 150, 200];
  useEffect(() => setSelectedWord(null), [props.selected?.id]);
  const inspectSelection = async () => {
    const selection = window.getSelection()?.toString().trim() ?? "";
    const word = selection.match(/[A-Za-z]+(?:['’-][A-Za-z]+)*/)?.[0];
    if (!word) return;
    setWordBusy(true);
    try {
      let found: LexiconEntry | null = null;
      for (const candidate of lexiconLookupCandidates(word)) {
        const results = await searchLexicon(candidate);
        found = results.find((entry) => entry.lemma.toLowerCase() === candidate) ?? null;
        if (found) break;
      }
      setSelectedWord(found);
    } catch (error) {
      props.onError(error);
    } finally {
      setWordBusy(false);
    }
  };
  const addSelectedWord = async () => {
    if (!selectedWord || selectedWord.inMyVocabulary) return;
    setWordBusy(true);
    try {
      await addToMyVocabulary(selectedWord.senseUid);
      setSelectedWord({ ...selectedWord, inMyVocabulary: true });
    } catch (error) {
      props.onError(error);
    } finally {
      setWordBusy(false);
    }
  };

  return (
    <section className="listening-page">
      <PageHeader
        actions={<Button onClick={props.onImport} variant="primary">{t("listening.import")}</Button>}
        eyebrow={t("app.name")}
        title={t("listening.title")}
      />
      {props.articles.length === 0 || !props.selected ? (
        <EmptyState><p>{t("listening.empty")}</p></EmptyState>
      ) : (
        <div className="listening-layout">
          <aside className="article-list" aria-label={t("listening.library")}>
            {props.articles.map((article) => (
              <button data-active={article.id === props.selected?.id} key={article.id} onClick={() => props.onSelect(article.id)} type="button">
                <strong>{article.title}</strong>
                <span>{t("listening.characters", { count: article.body.length })}</span>
              </button>
            ))}
          </aside>
          <article className="article-reader">
            <div className="article-reader__header">
              <div><p className="pb-eyebrow">{t("listening.nowPlaying")}</p><h2 className="pb-display">{props.selected.title}</h2></div>
              <Button onClick={() => setDeleteOpen(true)} variant="danger">{t("common.delete")}</Button>
            </div>
            <div className="listening-player">
              <div className="player-status"><span aria-hidden="true">●</span>{t(`listening.status.${props.playback}`)}</div>
              <div className="player-controls">
                {props.playback === "playing" ? <Button onClick={props.onPause}>{t("listening.pause")}</Button> : null}
                {props.playback === "paused" ? <Button onClick={props.onResume}>{t("listening.resume")}</Button> : null}
                {props.playback === "idle" ? <Button onClick={props.onPlay} variant="primary">{t("listening.play")}</Button> : null}
                <Button disabled={props.playback === "idle"} onClick={props.onStop}>{t("listening.stop")}</Button>
              </div>
              <div className="player-settings">
                <label>{t("settings.voice")}<SelectControl ariaLabel={t("settings.voice")} onValueChange={props.onVoiceChange} options={voiceOptions} value={props.voice} /></label>
                <fieldset><legend>{t("settings.speechRate")}</legend><div className="rate-options">{rateOptions.map((rate) => <button data-active={props.rate === rate} key={rate} onClick={() => props.onRateChange(rate)} type="button">{rate / 100}×</button>)}</div></fieldset>
              </div>
            </div>
            <p className="selection-hint">{t("listening.selectHint")}</p>
            <div className="article-body" lang="en" onMouseUp={() => void inspectSelection()} onTouchEnd={() => requestAnimationFrame(() => void inspectSelection())}>{props.selected.body}</div>
            {selectedWord ? (
              <aside className="listening-word" aria-live="polite">
                <div><strong>{selectedWord.lemma}</strong><span>{selectedWord.ipa}</span></div>
                <p lang="zh-CN">{selectedWord.zhGloss}</p>
                <Button disabled={wordBusy || selectedWord.inMyVocabulary} onClick={() => void addSelectedWord()} variant="primary">
                  {t(selectedWord.inMyVocabulary ? "listening.addedVocabulary" : "lexicon.addVocabulary")}
                </Button>
              </aside>
            ) : null}
          </article>
        </div>
      )}
      <ConfirmDialog
        cancelLabel={t("common.cancel")}
        confirmLabel={t("common.delete")}
        description={t("listening.deleteConfirm")}
        onConfirm={() => { setDeleteOpen(false); props.onDelete(); }}
        onOpenChange={setDeleteOpen}
        open={deleteOpen}
        title={props.selected?.title ?? t("listening.title")}
      />
    </section>
  );
}
