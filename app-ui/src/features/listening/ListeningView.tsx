import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";

import { EmptyState } from "../../design-system/components/EmptyState";
import { PageHeader } from "../../design-system/components/PageHeader";
import { ProgressStatus } from "../../design-system/components/ProgressStatus";
import { Button } from "../../design-system/primitives/Button";
import { ConfirmDialog } from "../../design-system/primitives/Dialogs";
import { SelectControl } from "../../design-system/primitives/SelectControl";
import {
  addToMyVocabulary,
  SPEECH_VOICES,
  type Article,
  type LexiconEntry,
  type SpeechVoice,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import { copyText } from "./clipboard";
import type { ArticleImportPhase, PlaybackState } from "./useListeningFlow";
import { findLexiconWord } from "./wordLookup";

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
  onTranslate: () => void;
  onRateChange: (rate: number) => void;
  onVoiceChange: (voice: SpeechVoice) => void;
  importPhase: ArticleImportPhase | null;
  playback: PlaybackState;
  rate: number;
  selected?: Article;
  translating: boolean;
  voice: SpeechVoice;
}

export function ListeningView(props: ListeningViewProps) {
  const { t } = useI18n();
  const [deleteOpen, setDeleteOpen] = useState(false);
  return (
    <section className="listening-page">
      <PageHeader
        actions={(
          <Button aria-busy={props.importPhase !== null} disabled={props.importPhase !== null} onClick={props.onImport} variant="primary">
            {props.importPhase ? t(`listening.importStatus.${props.importPhase}`) : t("listening.import")}
          </Button>
        )}
        eyebrow={t("app.name")}
        title={t("listening.title")}
      />
      {props.importPhase ? <ProgressStatus label={t(`listening.importStatus.${props.importPhase}`)} /> : null}
      {!props.selected ? <EmptyState><p>{t("listening.empty")}</p></EmptyState> : (
        <div className="listening-layout">
          <ArticleLibrary {...props} />
          <ArticleReader {...props} onRequestDelete={() => setDeleteOpen(true)} />
        </div>
      )}
      {props.selected && !props.selected.builtin ? <ConfirmDialog
        cancelLabel={t("common.cancel")}
        confirmLabel={t("common.delete")}
        description={t("listening.deleteConfirm")}
        onConfirm={() => { setDeleteOpen(false); props.onDelete(); }}
        onOpenChange={setDeleteOpen}
        open={deleteOpen}
        title={props.selected?.title ?? t("listening.title")}
      /> : null}
    </section>
  );
}

function ArticleLibrary(props: Pick<ListeningViewProps, "articles" | "onSelect" | "selected">) {
  const { t } = useI18n();
  return (
    <aside className="article-list" aria-label={t("listening.library")}>
      {props.articles.map((article) => (
        <button data-active={article.id === props.selected?.id} key={article.id} onClick={() => props.onSelect(article.id)} type="button">
          <strong>{article.title}</strong>
          <span>{article.builtin ? `${t("listening.builtin")} · ` : ""}{t("listening.characters", { count: article.body.length })}</span>
        </button>
      ))}
    </aside>
  );
}

function ArticleReader(props: ListeningViewProps & { onRequestDelete: () => void }) {
  const { t } = useI18n();
  const article = props.selected!;
  const lookup = useWordLookup(article.id, props.onError);
  return (
    <article className="article-reader">
      <div className="article-reader__header">
        <div><p className="pb-eyebrow">{t("listening.nowPlaying")}</p><h2 className="pb-display">{article.title}</h2></div>
        {article.builtin ? <span className="article-builtin-badge">{t("listening.builtin")}</span> : <Button onClick={props.onRequestDelete} variant="danger">{t("common.delete")}</Button>}
      </div>
      <ListeningPlayer {...props} />
      <BilingualArticle article={article} lookup={lookup} {...props} />
      {lookup.selectedWord ? <SelectedWordCard lookup={lookup} /> : null}
    </article>
  );
}

function ListeningPlayer(props: ListeningViewProps) {
  const { t } = useI18n();
  const voiceOptions = SPEECH_VOICES.map((value) => ({ label: t(`settings.voice.${value}`), value }));
  return (
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
        <RateOptions onChange={props.onRateChange} value={props.rate} />
      </div>
    </div>
  );
}

function RateOptions(props: { onChange: (rate: number) => void; value: number }) {
  const { t } = useI18n();
  return (
    <fieldset><legend>{t("settings.speechRate")}</legend><div className="rate-options">
      {[50, 100, 150, 200].map((rate) => <button data-active={props.value === rate} key={rate} onClick={() => props.onChange(rate)} type="button">{rate / 100}×</button>)}
    </div></fieldset>
  );
}

interface WordLookup {
  add: () => Promise<void>;
  busy: boolean;
  inspect: () => Promise<void>;
  selectedWord: LexiconEntry | null;
}

function BilingualArticle(props: ListeningViewProps & { article: Article; lookup: WordLookup }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState("");
  useEffect(() => setCopied(""), [props.article.id]);
  const copy = async (label: string, text: string) => {
    try {
      await copyText(text);
      setCopied(label);
    } catch (error) {
      props.onError(error);
    }
  };
  const translated = props.article.translatedBody;
  return (
    <section className="article-reading">
      <div className="article-reading__toolbar">
        <p className="selection-hint">{t("listening.selectHint")}</p>
        {translated ? <Button onClick={() => void copy(t("listening.copyBilingual"), bilingualText(props.article))}>{t("listening.copyBilingual")}</Button> : null}
      </div>
      <div className="article-bilingual">
        <MarkdownPane copyLabel={t("listening.copyEnglish")} language="en" markdown={props.article.body} onCopy={copy} onSelect={props.lookup.inspect} title={t("listening.englishOriginal")} />
        {translated ? (
          <MarkdownPane copyLabel={t("listening.copyChinese")} language="zh-CN" markdown={translated} onCopy={copy} title={t("listening.chineseTranslation")} />
        ) : (
          <TranslationEmpty onTranslate={props.onTranslate} translating={props.translating} />
        )}
      </div>
      {translated && !props.article.builtin ? <div className="translation-footer"><Button disabled={props.translating} onClick={props.onTranslate}>{t(props.translating ? "listening.translating" : "listening.retranslate")}</Button></div> : null}
      <p className="copy-status" role="status">{copied ? t("listening.copied", { item: copied }) : ""}</p>
    </section>
  );
}

function MarkdownPane(props: { copyLabel: string; language: string; markdown: string; onCopy: (label: string, text: string) => Promise<void>; onSelect?: () => Promise<void>; title: string }) {
  return (
    <section className="article-pane" lang={props.language}>
      <header><h3>{props.title}</h3><Button onClick={() => void props.onCopy(props.copyLabel, props.markdown)}>{props.copyLabel}</Button></header>
      <div className="article-body" onMouseUp={() => void props.onSelect?.()} onTouchEnd={() => requestAnimationFrame(() => void props.onSelect?.())}>
        <ReactMarkdown>{props.markdown}</ReactMarkdown>
      </div>
    </section>
  );
}

function TranslationEmpty(props: { onTranslate: () => void; translating: boolean }) {
  const { t } = useI18n();
  return (
    <section className="article-pane translation-empty" lang="zh-CN">
      <header><h3>{t("listening.chineseTranslation")}</h3></header>
      <div><p>{t("listening.translationEmpty")}</p><p>{t("listening.translationHint")}</p>
        <Button disabled={props.translating} onClick={props.onTranslate} variant="primary">{t(props.translating ? "listening.translating" : "listening.translate")}</Button>
      </div>
    </section>
  );
}

function SelectedWordCard(props: { lookup: WordLookup }) {
  const { t } = useI18n();
  const word = props.lookup.selectedWord!;
  return (
    <aside className="listening-word" aria-live="polite">
      <div><strong>{word.lemma}</strong><span>{word.ipa}</span></div>
      <p lang="zh-CN">{word.zhGloss}</p>
      <Button disabled={props.lookup.busy || word.inMyVocabulary} onClick={() => void props.lookup.add()} variant="primary">
        {t(word.inMyVocabulary ? "listening.addedVocabulary" : "lexicon.addVocabulary")}
      </Button>
    </aside>
  );
}

function useWordLookup(articleId: string, onError: (error: unknown) => void): WordLookup {
  const [selectedWord, setSelectedWord] = useState<LexiconEntry | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => setSelectedWord(null), [articleId]);
  const inspect = async () => {
    const word = window.getSelection()?.toString().trim().match(/[A-Za-z]+(?:['’-][A-Za-z]+)*/)?.[0];
    if (!word) return;
    setBusy(true);
    try {
      setSelectedWord(await findLexiconWord(word));
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  };
  const add = async () => {
    if (!selectedWord || selectedWord.inMyVocabulary) return;
    setBusy(true);
    try {
      await addToMyVocabulary(selectedWord.senseUid);
      setSelectedWord({ ...selectedWord, inMyVocabulary: true });
    } catch (error) {
      onError(error);
    } finally {
      setBusy(false);
    }
  };
  return { add, busy, inspect, selectedWord };
}

function bilingualText(article: Article): string {
  return `${article.body}\n\n---\n\n${article.translatedBody ?? ""}`;
}
