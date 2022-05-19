import type { OnLoadContext, OnLoadResult, OnResolveContext, OnResolveResult } from "@rspack/binding";

export interface RspackPlugin {
  onLoad(context: OnLoadContext): Promise<OnLoadResult | null | undefined>;
  onResolve(context: OnResolveContext): Promise<OnResolveResult | null | undefined>
}