import fs from 'node:fs';
import path from 'node:path';
import { type Edit, Lang, parse, type SgNode } from '@ast-grep/napi';
import type { Kinds, TypesMap } from '@ast-grep/napi/types/staticTypes';
import { defineConfig, type LibConfig, rsbuild, rspack } from '@rslib/core';
import prebundleConfig from './prebundle.config.mjs';

const merge = rsbuild.mergeRsbuildConfig;

const externalAlias: rsbuild.Rspack.Externals = ({ request }, callback) => {
  const { dependencies } = prebundleConfig;

  for (const item of dependencies) {
    const depName = typeof item === 'string' ? item : item.name;
    if (new RegExp(`^${depName}$`).test(request!)) {
      return callback(undefined, `../compiled/${depName}/index.js`);
    }
  }

  if (new RegExp(/^tinypool$/).test(request!)) {
    return callback(undefined, '../compiled/tinypool/dist/index.js');
  }

  return callback();
};

const commonLibConfig: LibConfig = {
  format: 'cjs',
  syntax: ['es2023'],
  source: {
    define: {
      WEBPACK_VERSION: JSON.stringify(require('./package.json').webpackVersion),
      RSPACK_VERSION: JSON.stringify(require('./package.json').version),
      IS_BROWSER: JSON.stringify(false),
    },
  },
  output: {
    externals: ['@rspack/binding/package.json', externalAlias],
    minify: {
      js: true,
      jsOptions: {
        minimizerOptions: {
          // preserve variable name and disable minify for easier debugging
          mangle: false,
          minify: false,
          compress: {
            // enable to compress import.meta.url shims in top level scope
            toplevel: true,
            // keep debugger so we can debug in the debug terminal without need to search in minified dist
            drop_debugger: false,
          },
        },
      },
    },
  },
};

const mfRuntimePlugin: rsbuild.RsbuildPlugin = {
  name: 'mf-runtime',
  setup(api) {
    api.onAfterBuild(async () => {
      const { swc } = rspack.experiments;
      const runtime = await fs.promises.readFile(
        path.resolve(
          __dirname,
          'src/runtime/moduleFederationDefaultRuntime.js',
        ),
        'utf-8',
      );

      const { code: downgradedRuntime } = await swc.transform(runtime, {
        jsc: {
          target: 'es2015',
        },
      });

      const minimizedRuntime = await swc.minify(downgradedRuntime, {
        compress: false,
        mangle: false,
        ecma: 2015,
      });

      await fs.promises.writeFile(
        path.resolve(__dirname, 'dist/moduleFederationDefaultRuntime.js'),
        minimizedRuntime.code,
      );
    });
  },
};

const codmodPlugin: rsbuild.RsbuildPlugin = {
  name: 'codmod',
  setup(api) {
    /**
     * Replaces `@rspack/binding` to code that reads env `RSPACK_BINDING` as the custom binding.
     */
    function replaceBinding(root: SgNode<TypesMap, Kinds<TypesMap>>): Edit[] {
      const target = `module1.exports = require("@rspack/binding");`;
      const binding = root.find(target);
      if (binding === null) {
        throw new Error('Cannot find binding require statement: ' + target);
      }
      return [
        binding.replace(
          `module1.exports = require(process.env.RSPACK_BINDING ? process.env.RSPACK_BINDING : "@rspack/binding");`,
        ),
      ];
    }

    api.onAfterBuild(async () => {
      const dist = fs.readFileSync(
        require.resolve(path.resolve(__dirname, 'dist/index.js')),
        'utf-8',
      );
      const root = parse(Lang.JavaScript, dist).root();
      const edits = [...replaceBinding(root)];

      fs.writeFileSync(
        require.resolve(path.resolve(__dirname, 'dist/index.js')),
        root.commitEdits(edits),
      );
    });
  },
};

export default defineConfig({
  plugins: [mfRuntimePlugin, codmodPlugin],
  lib: [
    merge(commonLibConfig, {
      dts: {
        build: true,
      },
      source: {
        entry: {
          index: './src/index.ts',
        },
        tsconfigPath: './tsconfig.build.json',
      },
      output: {
        externals: [externalAlias, './moduleFederationDefaultRuntime.js'],
      },
      footer: {
        // make default export in cjs work
        js: 'module.exports = __webpack_exports__.default;',
      },
    }),
    merge(commonLibConfig, {
      source: {
        entry: {
          cssExtractLoader: './src/builtin-plugin/css-extract/loader.ts',
        },
      },
    }),
    merge(commonLibConfig, {
      syntax: 'es2015',
      source: {
        entry: {
          cssExtractHmr: './src/runtime/cssExtractHmr.ts',
        },
      },
    }),
    merge(commonLibConfig, {
      source: {
        entry: {
          worker: './src/loader-runner/worker.ts',
        },
      },
      footer: {
        // make default export in cjs work
        js: 'module.exports = __webpack_exports__.default;',
      },
    }),
  ],
});
