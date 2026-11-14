import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  Object.entries({
    plain: './plain.mjs',
    rslib: './index.js',
    wrapped: ['./wrapped.mjs', './hoisted.mjs'],
    alias: ['./first-alias.mjs', './second-alias.mjs'],
    namespace: './namespace.mjs',
  }).map(([name, entry]) => {
    const rslib = name !== 'plain';
    return defineConfig({
      name,
      target: 'node',
      entry,
      externalsType: 'module',
      externals: [
        './ext.mjs',
        './middle-side.mjs',
        './other.mjs',
        './side.mjs',
        ({ request, contextInfo }, callback) => {
          if (request !== 'alias') return callback();
          const target = contextInfo?.issuer.endsWith('first-alias.mjs')
            ? './ext.mjs'
            : './middle-side.mjs';
          callback(null, target);
        },
      ],
      output: {
        filename: `${name}/[name].mjs`,
        chunkFilename: `${name}/[name].mjs`,
      },
      plugins: rslib ? [new rspack.experiments.RslibPlugin()] : [],
      optimization: {
        minimize: false,
        runtimeChunk: false,
      },
    });
  }),
);
