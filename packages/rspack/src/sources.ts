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
export type * from 'webpack-sources';

export {
  RspackConcatSource as ConcatSource,
  RspackReplaceSource as ReplaceSource,
} from './util/source';
