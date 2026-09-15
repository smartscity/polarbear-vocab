import { useState } from "react";

import { Button } from "../../design-system/primitives/Button";
import type { CollectionSpec, HomeDto } from "../../lib/commands";
import { useI18n } from "../../lib/i18n";

type SessionCollection = "unseen" | "mistakes" | "all";

interface SessionSetupViewProps {
  home: HomeDto;
  initialCollection?: SessionCollection;
  onCancel: () => void;
  onStart: (spec: CollectionSpec, limit: number) => void;
}

export function SessionSetupView(props: SessionSetupViewProps) {
  const { t } = useI18n();
  const [collection, setCollection] = useState<SessionCollection>(props.initialCollection ?? "unseen");
  const [limit, setLimit] = useState(20);
  const options: Array<{ type: SessionCollection; count: number }> = [
    { type: "unseen", count: props.home.progress.unseen },
    { type: "mistakes", count: props.home.mistakeBuckets.atLeastOne },
    { type: "all", count: props.home.progress.total },
  ];
  const selected = options.find((option) => option.type === collection);
  const spec = sessionSpec(collection, props.home.dataset.id);

  return (
    <section className="session-setup">
      <button className="pb-back" onClick={props.onCancel} type="button">← {t("common.back")}</button>
      <p className="pb-eyebrow">{t("session.eyebrow")}</p>
      <h1 className="pb-display">{props.home.dataset.name}</h1>
      <div aria-label={t("session.collection")} className="session-collections" role="radiogroup">
        {options.map((option) => (
          <button
            aria-checked={collection === option.type}
            disabled={option.count === 0}
            key={option.type}
            onClick={() => setCollection(option.type)}
            role="radio"
            type="button"
          >
            <span>{t(`session.${option.type}`)}</span><strong>{option.count.toLocaleString()}</strong>
          </button>
        ))}
      </div>
      <label className="session-size">
        <span>{t("session.size")}</span>
        <select onChange={(event) => setLimit(Number(event.target.value))} value={limit}>
          {[10, 20, 50].map((count) => <option key={count} value={count}>{t("session.words", { count })}</option>)}
        </select>
      </label>
      <Button disabled={!selected || selected.count === 0} onClick={() => props.onStart(spec, limit)} variant="primary">
        {t("session.start")}
      </Button>
    </section>
  );
}

function sessionSpec(collection: SessionCollection, datasetId: string): CollectionSpec {
  if (collection === "unseen") return { type: "unseen", datasetId };
  if (collection === "mistakes") return { type: "wrong", datasetId, minWrongCount: 1 };
  return { type: "dataset", datasetId };
}
