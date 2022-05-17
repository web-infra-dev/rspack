import type { OnLoadContext, OnLoadResult } from "@rspack/binding";

export interface RspackPlugin {
  onLoad(context: OnLoadContext): Promise<OnLoadResult>;
}