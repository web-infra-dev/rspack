import type { Configuration } from '@rspack/core';
import rspack from '@rspack/core';

export default {
  entry: './index.js',
  context: import.meta.dirname,
  // use production mod to make sure
  // the persistent cache will write to disk
  mode: 'production',
  plugins: [new rspack.HtmlRspackPlugin()],
  cache: {
    type: 'persistent',
  },
} satisfies Configuration;
