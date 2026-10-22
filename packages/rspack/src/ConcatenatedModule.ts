import binding from '@rspack/binding';
import type { Source } from 'webpack-sources';
import { SourceAdapter } from './util/source';
import { MODULE_IDENTIFIER_SYMBOL } from './util/symbols';

Object.defineProperty(binding.ConcatenatedModule.prototype, 'identifier', {
  enumerable: true,
  configurable: true,
  value(this: binding.ConcatenatedModule): string {
    // @ts-expect-error
    return this[MODULE_IDENTIFIER_SYMBOL];
  },
});
Object.defineProperty(binding.ConcatenatedModule.prototype, 'originalSource', {
  enumerable: true,
  configurable: true,
  value(this: binding.ConcatenatedModule) {
    const originalSource = this._originalSource();
    if (originalSource) {
      return SourceAdapter.fromBinding(originalSource);
    }
    return null;
  },
});
Object.defineProperty(binding.ConcatenatedModule.prototype, 'emitFile', {
  enumerable: true,
  configurable: true,
  value(
    this: binding.ConcatenatedModule,
    filename: string,
    source: Source,
    assetInfo?: binding.AssetInfo,
  ) {
    return this._emitFile(filename, SourceAdapter.toBinding(source), assetInfo);
  },
});

export { ConcatenatedModule } from '@rspack/binding';
