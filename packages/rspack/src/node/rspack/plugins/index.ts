import type { OnLoadContext, OnLoadResult, OnResolveContext, OnResolveResult, ExternalObject } from "@rspack/binding";

export interface RspackPlugin {
  onLoad(context: OnLoadContext, rspack: ExternalObject<any>): Promise<OnLoadResult | void>;
  onResolve(context: OnResolveContext, rspack: ExternalObject<any>): Promise<OnResolveResult | void>
}

