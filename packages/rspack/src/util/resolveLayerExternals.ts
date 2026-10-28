/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/cleverMerge.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type {
  ExternalItem,
  ExternalItemObjectKnown,
  ExternalItemObjectUnknown,
} from '../config/types';
import { cachedCleverMerge } from './cleverMerge';

export const resolveLayerExternals = <T>(item: T, layer: string | null): T => {
  if (typeof item !== 'object' || item === null || !('byLayer' in item)) {
    return item;
  }
  const { byLayer, ...remaining } = item as ExternalItemObjectKnown;
  if (typeof byLayer === 'object') {
    const key = String(layer);
    if (key in byLayer) {
      return mergeExternalObjects(remaining, byLayer[key]) as T;
    }
    if ('default' in byLayer) {
      return mergeExternalObjects(remaining, byLayer.default) as T;
    }
    return remaining as T;
  }
  if (typeof byLayer === 'function') {
    return mergeExternalObjects(
      remaining,
      resolveLayerExternals(byLayer(layer), layer),
    ) as T;
  }
  return item;
};

function mergeExternalObjects(
  base: ExternalItemObjectUnknown,
  overrides: ExternalItem | undefined,
): ExternalItem | undefined {
  if (typeof overrides !== 'object' || overrides === null) {
    return overrides === undefined ? base : overrides;
  }
  return {
    ...base,
    ...Object.fromEntries(
      Object.entries(overrides).map(([request, value]) => [
        request,
        cachedCleverMerge(
          Object.hasOwn(base, request) ? base[request] : undefined,
          value,
        ),
      ]),
    ),
  };
}
