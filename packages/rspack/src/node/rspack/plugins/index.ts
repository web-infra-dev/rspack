import type { OnLoadResult, OnResolveResult } from '@rspack/binding';

export interface RspackPlugin {
  name: string;
  buildStart?(): Promise<void>;
  load?(id: string): Promise<OnLoadResult | void>;
  resolve?(source: string, importer: string | undefined): Promise<OnResolveResult | void>;
  buildEnd?(): Promise<void>;
}
