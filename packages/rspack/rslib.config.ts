import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { type Edit, Lang, parse, type SgNode } from '@ast-grep/napi';
import type { Kinds, TypesMap } from '@ast-grep/napi/types/staticTypes';
import {
  defineConfig,
  type LibConfig,
  type RsbuildPlugin,
  type Rspack,
  rsbuild,
  rspack,
} from '@rslib/core';
import packageJson from './package.json' with { type: 'json' };
import prebundleConfig from './prebundle.config.mjs';

const require = createRequire(import.meta.url);

const merge = rsbuild.mergeRsbuildConfig;

const externalAlias: Rspack.Externals = ({ request }, callback) => {
  const { dependencies } = prebundleConfig;

  for (const item of dependencies) {
    const depName = typeof item === 'string' ? item : item.name;
    if (new RegExp(`^${depName}$`).test(request!)) {
      return callback(
        undefined,
        `node-commonjs ../compiled/${depName}/index.js`,
      );
    }
  }

  if (new RegExp(/^tinypool$/).test(request!)) {
    return callback(undefined, '../compiled/tinypool/dist/index.js');
  }

  return callback();
};

const commonLibConfig: LibConfig = {
  format: 'esm',
  syntax: ['es2023'],
  source: {
    define: {
      WEBPACK_VERSION: JSON.stringify(packageJson.webpackVersion),
      RSPACK_VERSION: JSON.stringify(packageJson.version),
      IS_BROWSER: JSON.stringify(false),
    },
  },
  output: {
    externals: [
      {
        '@rspack/binding': 'node-commonjs @rspack/binding',
      },
      './moduleFederationDefaultRuntime.js',
      '@rspack/binding/package.json',
      externalAlias,
    ],
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

const mfRuntimePlugin: RsbuildPlugin = {
  name: 'mf-runtime',
  setup(api) {
    api.onAfterBuild(async () => {
      const { swc } = rspack.experiments;
      const runtime = await fs.promises.readFile(
        path.resolve(
          import.meta.dirname,
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
        path.resolve(
          import.meta.dirname,
          'dist/moduleFederationDefaultRuntime.js',
        ),
        minimizedRuntime.code,
      );
    });
  },
};

const codmodPlugin: RsbuildPlugin = {
  name: 'codmod',
  setup(api) {
    /**
     * Replaces `@rspack/binding` to code that reads env `RSPACK_BINDING` as the custom binding.
     */
    function replaceBinding(root: SgNode<TypesMap, Kinds<TypesMap>>): Edit[] {
      const edits: Edit[] = [];

      // Pattern 1: let binding_namespaceObject = __rspack_createRequire_require("@rspack/binding");
      const pattern1 = `let binding_namespaceObject = __rspack_createRequire_require("@rspack/binding");`;
      const binding1 = root.find(pattern1);
      if (binding1) {
        edits.push(
          binding1.replace(
            `let binding_namespaceObject = __rspack_createRequire_require(process.env.RSPACK_BINDING ? process.env.RSPACK_BINDING : "@rspack/binding");`,
          ),
        );
      }

      // Pattern 2: let instanceBinding = Compiler_require('@rspack/binding');
      const pattern2 = `let instanceBinding = Compiler_require('@rspack/binding');`;
      const binding2 = root.find(pattern2);
      if (binding2) {
        edits.push(
          binding2.replace(
            `let instanceBinding = Compiler_require(process.env.RSPACK_BINDING ? process.env.RSPACK_BINDING : '@rspack/binding');`,
          ),
        );
      }

      if (edits.length === 0) {
        throw new Error(
          'Cannot find any binding require statements to replace',
        );
      }

      return edits;
    }

    api.onAfterBuild(async () => {
      const dist = fs.readFileSync(
        require.resolve(path.resolve(import.meta.dirname, 'dist/index.js')),
        'utf-8',
      );
      const root = parse(Lang.JavaScript, dist).root();
      const edits = [...replaceBinding(root)];

      fs.writeFileSync(
        require.resolve(path.resolve(import.meta.dirname, 'dist/index.js')),
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
      redirect: {
        dts: {
          extension: true,
        },
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
      tools: {
        rspack: {
          plugins: [
            new rspack.BannerPlugin({
              // make require esm default export compatible with commonjs
              banner: `export { src_rspack_0 as 'module.exports' }`,
              stage: rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE + 1,
              raw: true,
              footer: true,
              include: /index\.js$/,
            }),
          ],
        },
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
    }),
  ],
});
