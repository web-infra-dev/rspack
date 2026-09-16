import { sharing } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ProvideSharedPlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  plugins: [
    new ProvideSharedPlugin({
      shareScope: 'eagerOverrideNonEager',
      provides: {
        common: {
          shareKey: 'common',
          eager: true,
        },
      },
    }),
    new ProvideSharedPlugin({
      shareScope: 'nonEagerDontOverrideEager',
      provides: {
        uncommon: {
          shareKey: 'uncommon',
        },
      },
    }),
    new ProvideSharedPlugin({
      shareScope: 'newerNonEager',
      provides: {
        uncommon: {
          shareKey: 'uncommon',
        },
      },
    }),
    new ProvideSharedPlugin({
      shareScope: 'newerEager',
      provides: {
        common: {
          shareKey: 'common',
          eager: true,
        },
      },
    }),
  ],
};
