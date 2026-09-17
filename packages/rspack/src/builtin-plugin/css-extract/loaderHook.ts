import type { LoaderContext } from '../../config';
import type { Module } from '../../Module';
import { type CssExtractPluginData, pluginSymbol } from './utils';

export default function loaderHook(
  loaderContext: LoaderContext,
  _module: Module,
  options: CssExtractPluginData,
): void {
  (loaderContext as unknown as Record<symbol, CssExtractPluginData>)[
    pluginSymbol
  ] = { runtime: options.runtime };
}
