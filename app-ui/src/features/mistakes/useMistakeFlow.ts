import { useCallback, useState } from "react";

import { listWrongWords, type WrongWord } from "../../lib/commands";

export function useMistakeFlow(
  datasetId: string,
  onError: (error: unknown) => void,
  onShow: () => void,
) {
  const [words, setWords] = useState<WrongWord[]>([]);
  const [filter, setFilter] = useState({ minimum: 1, lastWrongOnly: false });
  const show = useCallback(async (minimum: number, lastWrongOnly: boolean) => {
    try {
      setWords(await listWrongWords(datasetId || undefined, minimum, lastWrongOnly));
      setFilter({ minimum, lastWrongOnly });
      onShow();
    } catch (error) {
      onError(error);
    }
  }, [datasetId, onError, onShow]);
  return { filter, show, words };
}
