import type { OnLoadContext, OnLoadResult, OnResolveContext, OnResolveResult } from "@rspack/binding";

export interface RspackPlugin {
  onLoad(context: OnLoadContext): Promise<OnLoadResult>;
  onResolve(context: OnResolveContext): Promise<OnResolveResult>
}