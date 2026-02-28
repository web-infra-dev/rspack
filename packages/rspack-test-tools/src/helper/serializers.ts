import type { Plugins } from 'pretty-format';
import { normalizeDiff } from './expect/diff';
import { normalizeDignostics, normalizeError } from './expect/error';
import { normalizePlaceholder } from './expect/placeholder';
import { normalizeStats } from './expect/rspack';

export const serializers: Plugins = [
  {
    test(received) {
      return typeof received === 'string';
    },
    print(received) {
      return normalizePlaceholder((received as string).trim());
    },
  },
  // for diff
  {
    test(received) {
      return received?.constructor?.name === 'RspackTestDiff';
    },
    print(received, next) {
      return next(normalizeDiff(received as { value: string }));
    },
  },
  // for errors
  {
    test(received) {
      return received?.constructor?.name === 'RspackStatsDiagnostics';
    },
    print(received, next) {
      return next(
        normalizeDignostics(received as { errors: Error[]; warnings: Error[] }),
      );
    },
  },
  {
    test(received) {
      return typeof received?.message === 'string';
    },
    print(received, next) {
      return next(normalizeError(received as Error));
    },
  },
  // for stats
  {
    test(received) {
      return received?.constructor?.name === 'RspackStats';
    },
    print(received, next) {
      return next(normalizeStats(received as { value: string }));
    },
  },
];
