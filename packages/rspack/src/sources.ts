export {
  CachedSource,
  CompatSource,
  OriginalSource,
  PrefixSource,
  RawSource,
  SizeOnlySource,
  Source,
  SourceMapSource,
  util,
} from 'webpack-sources';
export type {
  CachedData,
  ConcatSourceChild,
  GeneratedSourceInfo,
  HashLike,
  MapOptions,
  OnChunk,
  OnName,
  OnSource,
  RawSourceMap,
  Replacement,
  SourceAndMap,
  SourceLike,
  SourceValue,
  StreamChunksOptions,
} from 'webpack-sources';

export {
  RspackConcatSource as ConcatSource,
  RspackReplaceSource as ReplaceSource,
} from './util/source';
