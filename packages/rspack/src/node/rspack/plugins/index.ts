import type { OnLoadContext, OnLoadResult, OnResolveContext, OnResolveResult, ExternalObject } from '@rspack/binding';

export interface RspackPlugin {
  name: string;
  onLoad(context: OnLoadContext): Promise<OnLoadResult | void>;
  onResolve(context: OnResolveContext): Promise<OnResolveResult | void>;
}
