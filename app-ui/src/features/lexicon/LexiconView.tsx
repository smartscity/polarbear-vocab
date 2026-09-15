import type { FormEvent } from "react";

import { EmptyState } from "../../design-system/components/EmptyState";
import { PageHeader } from "../../design-system/components/PageHeader";
import { Button } from "../../design-system/primitives/Button";
import type { LexiconEntry } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

interface LexiconViewProps {
  busy: boolean;
  onPractice: (senseUid: string) => void;
  onQueryChange: (query: string) => void;
  onSearch: () => void;
  onToggleVocabulary: (entry: LexiconEntry) => void;
  query: string;
  results: LexiconEntry[];
  searched: boolean;
}

export function LexiconView(props: LexiconViewProps) {
  const { t } = useI18n();
  const submit = (event: FormEvent) => {
    event.preventDefault();
    props.onSearch();
  };
  return (
    <section className="lexicon-page">
      <PageHeader eyebrow={t("app.knowledgeModule")} title={t("lexicon.title")} />
      <form className="lexicon-search" onSubmit={submit} role="search">
        <input
          aria-label={t("lexicon.search")}
          autoCapitalize="none"
          autoCorrect="off"
          className="pb-input"
          maxLength={80}
          onChange={(event) => props.onQueryChange(event.currentTarget.value)}
          placeholder={t("lexicon.placeholder")}
          value={props.query}
        />
        <Button disabled={props.busy || !props.query.trim()} type="submit" variant="primary">
          {t("lexicon.search")}
        </Button>
      </form>
      {props.results.length > 0 ? (
        <div className="lexicon-results">
          {props.results.map((entry) => (
            <LexiconCard
              busy={props.busy}
              entry={entry}
              key={entry.senseUid}
              onPractice={props.onPractice}
              onToggleVocabulary={props.onToggleVocabulary}
            />
          ))}
        </div>
      ) : (
        <EmptyState><p>{t(props.searched ? "lexicon.noResults" : "lexicon.hint")}</p></EmptyState>
      )}
    </section>
  );
}

function LexiconCard(props: {
  busy: boolean;
  entry: LexiconEntry;
  onPractice: (senseUid: string) => void;
  onToggleVocabulary: (entry: LexiconEntry) => void;
}) {
  const { t } = useI18n();
  const entry = props.entry;
  return (
    <article className="lexicon-card">
      <header>
        <div><h2 className="pb-display">{entry.lemma}</h2><p>{entry.ipa}</p></div>
        <span>{entry.partOfSpeech}</span>
      </header>
      <p className="lexicon-gloss" lang="zh-CN">{entry.zhGloss}</p>
      {entry.exampleEn ? <p className="lexicon-example" lang="en">{entry.exampleEn}</p> : null}
      <div className="lexicon-tags">{entry.datasetNames.map((name) => <span key={name}>{name}</span>)}</div>
      <dl className="lexicon-stats">
        <div><dt>{t("lexicon.answered")}</dt><dd>{entry.attemptCount}</dd></div>
        <div><dt>{t("stats.correct")}</dt><dd>{entry.correctCount}</dd></div>
        <div><dt>{t("stats.wrong")}</dt><dd>{entry.wrongCount}</dd></div>
      </dl>
      <footer>
        <Button disabled={props.busy} onClick={() => props.onToggleVocabulary(entry)}>
          {t(entry.inMyVocabulary ? "lexicon.removeVocabulary" : "lexicon.addVocabulary")}
        </Button>
        <Button onClick={() => props.onPractice(entry.senseUid)} variant="primary">{t("lexicon.practice")}</Button>
      </footer>
    </article>
  );
}
