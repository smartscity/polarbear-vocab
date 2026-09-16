declare module "@browsermt/bergamot-translator/translator.js" {
  export interface TranslationRequest {
    from: string;
    to: string;
    text: string;
    html?: boolean;
    priority?: number;
  }

  export interface TranslationResponse {
    request: TranslationRequest;
    target: { text: string };
  }

  export interface ModelFile {
    name: string;
    expectedSha256Hash?: string;
  }

  export interface TranslationModel {
    from: string;
    to: string;
    files: Record<string, ModelFile | Record<string, unknown> | undefined>;
  }

  export class TranslatorBacking {
    constructor(options?: Record<string, unknown>);
    loadModelRegistery(): Promise<TranslationModel[]>;
    fetch(url: string, checksum?: string, options?: { signal?: AbortSignal }): Promise<ArrayBuffer>;
  }

  export class BatchTranslator {
    constructor(options?: Record<string, unknown>, backing?: TranslatorBacking);
    translate(request: TranslationRequest): Promise<TranslationResponse>;
    delete(): Promise<void>;
  }
}
