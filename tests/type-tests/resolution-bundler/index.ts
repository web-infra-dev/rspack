import rspack, { type RspackOptions } from '@rspack/core';
import '@rspack/core/module';
import { defineConfig, definePlugin, type Configuration } from '@rspack/cli';

const plugin = definePlugin({
  apply(compiler) {
    compiler.hooks.done.tap('type-test', () => undefined);
  },
});

const config: RspackOptions = {
  entry: './src/index.js',
  plugins: [
    plugin,
    new rspack.DefinePlugin({
      __TYPE_TEST__: JSON.stringify(true),
    }),
  ],
  devServer: {
    proxy: [
      {
        context: ['/api'],
        target: 'http://localhost:3000',
      },
    ],
  },
};

type GlobModule = {
  default: string;
};

const eagerGlobModules = import.meta.glob<GlobModule>('./dir/*.js', {
  eager: true,
});
eagerGlobModules['./dir/foo.js'].default.toUpperCase();

const lazyGlobModules = import.meta.glob<GlobModule>('./dir/*.js');
lazyGlobModules['./dir/foo.js']().then((mod) => mod.default.toUpperCase());

const eagerDefaultGlobModules = import.meta.glob<string>('./dir/*.js', {
  eager: true,
  import: 'default',
});
eagerDefaultGlobModules['./dir/foo.js'].toUpperCase();

const lazyDefaultGlobModules = import.meta.glob<string>('./dir/*.js', {
  import: 'default',
});
lazyDefaultGlobModules['./dir/foo.js']().then((mod) => mod.toUpperCase());

import.meta.rspackPublicPath.toUpperCase();
import.meta.rspackBaseUri.toUpperCase();
Object.keys(import.meta.rspackShareScopes).forEach((scope) =>
  scope.toUpperCase(),
);
import.meta.rspackInitSharing('default').then(() => undefined);
import.meta.rspackNonce.toUpperCase();
import.meta.rspackUniqueId.toUpperCase();
import.meta.rspackVersion.toUpperCase();
import.meta.rspackHash.toUpperCase();

const multiGlobModules = import.meta.glob<GlobModule>(
  ['./dir/*.js', '!**/bar.js'] as const,
  {
    eager: true,
    query: {
      raw: true,
    },
    base: './base',
    exhaustive: true,
  },
);
multiGlobModules['./dir/foo.js'].default.toUpperCase();

export const cliConfig: Configuration = defineConfig(config);
