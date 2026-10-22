import binding from '@rspack/binding';
import type { Source } from 'webpack-sources';
import { SourceAdapter } from './util/source';

Object.defineProperty(binding.Sources.prototype, 'get', {
  enumerable: true,
  configurable: true,
  value(this: binding.Sources, sourceType: string) {
    const originalSource = this._get(sourceType);
    if (originalSource) {
      return SourceAdapter.fromBinding(originalSource);
    }
    return null;
  },
});

declare module '@rspack/binding' {
  interface Sources {
    get(sourceType: string): Source | null;
  }
}
