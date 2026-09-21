import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";

import { Button } from "../../design-system/primitives/Button";
import { Modal } from "../../design-system/primitives/Dialogs";
import {
  addToMyVocabulary,
  takeMacosServiceRequests,
  type LexiconEntry,
  type MacosServiceRequest,
} from "../../lib/commands";
import { useI18n } from "../../lib/i18n";
import { translateEnglishMarkdown } from "../listening/articleTranslation";
import { copyText } from "../listening/clipboard";
import { findLexiconWord } from "../listening/wordLookup";

interface ServiceResult {
  entry?: LexiconEntry;
  translation?: string;
}

export function MacosServicePanel({ onError }: { onError: (error: unknown) => void }) {
  const { t } = useI18n();
  const [request, setRequest] = useState<MacosServiceRequest | null>(null);
  const [result, setResult] = useState<ServiceResult | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const drain = async () => {
      const latest = (await takeMacosServiceRequests()).at(-1);
      if (latest) await handleRequest(latest, setRequest, setResult, setBusy);
    };
    void drain().catch(onError);
    const unlisten = listen("macos-service-pending", () => void drain().catch(onError));
    return () => { void unlisten.then((stop) => stop()); };
  }, [onError]);

  const close = () => {
    setRequest(null);
    setResult(null);
  };
  const copy = async (value: string) => {
    try {
      await copyText(value);
    } catch (error) {
      onError(error);
    }
  };
  return (
    <Modal
      description={request?.action === "translate" ? t("macosService.translateHint") : t("macosService.vocabularyHint")}
      onOpenChange={(open) => { if (!open) close(); }}
      open={request !== null}
      title={request?.action === "translate" ? t("macosService.translateTitle") : t("macosService.vocabularyTitle")}
    >
      {request ? <div className="macos-service-panel">
        <section><h3>{t("macosService.selection")}</h3><p>{request.text}</p></section>
        {busy ? <p role="status">{t("common.loading")}</p> : <ServiceOutput request={request} result={result} />}
        <div className="pb-dialog-actions">
          {result?.translation ? <Button onClick={() => void copy(result.translation!)}>{t("macosService.copyTranslation")}</Button> : null}
          <Button onClick={close} variant="primary">{t("common.close")}</Button>
        </div>
      </div> : null}
    </Modal>
  );
}

function ServiceOutput(props: { request: MacosServiceRequest; result: ServiceResult | null }) {
  const { t } = useI18n();
  if (props.request.action === "translate") {
    return <section className="macos-service-result" lang="zh-CN"><h3>{t("macosService.translation")}</h3><ReactMarkdown>{props.result?.translation ?? ""}</ReactMarkdown></section>;
  }
  const entry = props.result?.entry;
  if (!entry) return <p role="status">{t("macosService.wordNotFound")}</p>;
  return <section className="macos-service-result"><h3>{entry.lemma} <span>{entry.ipa}</span></h3><p lang="zh-CN">{entry.zhGloss}</p><p role="status">{t("macosService.wordAdded")}</p></section>;
}

async function handleRequest(
  request: MacosServiceRequest,
  setRequest: (value: MacosServiceRequest) => void,
  setResult: (value: ServiceResult | null) => void,
  setBusy: (value: boolean) => void,
) {
  setRequest(request);
  setResult(null);
  setBusy(true);
  try {
    if (request.action === "translate") {
      setResult({ translation: await translateEnglishMarkdown(request.text) });
      return;
    }
    const selectedWord = request.text.match(/[A-Za-z]+(?:['’-][A-Za-z]+)*/)?.[0];
    const entry = selectedWord ? await findLexiconWord(selectedWord) : null;
    if (entry && !entry.inMyVocabulary) await addToMyVocabulary(entry.senseUid);
    setResult({ entry: entry ? { ...entry, inMyVocabulary: true } : undefined });
  } finally {
    setBusy(false);
  }
}
