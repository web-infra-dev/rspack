import { defineConfig, definePlugin } from '@rspack/cli';

export default ['fixed', 'auto', 'worker'].map((kind, index) =>
  defineConfig({
    mode: 'development',
    target: 'web',
    devtool: false,
    output: {
      publicPath: kind === 'fixed' ? '/initial/' : 'auto',
      filename: `bundle${index}.js`,
    },
    plugins: [
      definePlugin((compiler) => {
        const {
          RuntimeGlobals: g,
          RuntimeModule,
          DefinePlugin,
        } = compiler.rspack;
        const rspackMode =
          compiler.options.experiments.runtimeMode === 'rspack';
        new DefinePlugin({ PATH_KIND: JSON.stringify(kind) }).apply(compiler);
        compiler.hooks.thisCompilation.tap('SourceHooks', (compilation) => {
          const hasEntry = (
            chunk: Parameters<typeof compilation.chunkGraph.getChunkModules>[0],
          ) => compilation.chunkGraph.getChunkModules(chunk).length > 0;
          class ExtensionPaths extends RuntimeModule {
            constructor() {
              super('extension paths', RuntimeModule.STAGE_TRIGGER);
            }
            generate() {
              const graphHasEntry = this.chunk && hasEntry(this.chunk);
              return `
            __webpack_require__.webExtRt = {
              previous: __webpack_require__.p,
              hasEntry: ${compilation.runtimeTemplate.returningFunction(JSON.stringify(graphHasEntry))}
            };
            __webpack_require__.b = 'https://extension.test/';
            __webpack_require__.p = new URL('assets/', __webpack_require__.b).href;
          `;
            }
          }
          const requirements =
            compilation.hooks.additionalTreeRuntimeRequirements;
          // tramvai supplies the webpack hook's third argument for a JS consumer.
          requirements.intercept({
            register(tap) {
              if (tap.name !== 'SourceHooks') return tap;
              return {
                ...tap,
                fn: (chunk, set) =>
                  tap.fn(chunk, set, { chunkGraph: compilation.chunkGraph }),
              };
            },
          });
          requirements.tap(
            'SourceHooks',
            (
              _chunk,
              set,
              context?: { chunkGraph: typeof compilation.chunkGraph },
            ) => {
              if (context?.chunkGraph !== compilation.chunkGraph)
                throw new Error('missing chunk graph');
              set.add(g.global);
              set.add(g.publicPath);
              set.add(g.baseURI);
            },
          );
          compilation.hooks.runtimeRequirementInTree
            .for(g.publicPath)
            .tap('ExtensionPaths', (chunk) => {
              compilation.addRuntimeModule(chunk, new ExtensionPaths());
            });
          compilation.hooks.runtimeModule.tap('ChunkGraphCompat', (mod) => {
            const graph = Reflect.get(mod, 'chunkGraph');
            if (graph && !graph.getChunkConditionMap) {
              graph.getChunkConditionMap = (
                chunk: Parameters<typeof hasEntry>[0],
              ) => ({ [chunk.id!]: hasEntry(chunk) });
            }
          });
          compilation.hooks.runtimeModule.tap('SourceHooks', (mod, chunk) => {
            if (!mod.source) return;
            if (mod.constructor.name === 'PublicPathRuntimeModule') {
              // kmi / the public replacement for Umi's _cachedGeneratedCode.
              mod.source.source = Buffer.from(
                '__webpack_require__.p = "/configured/";',
              );
            } else if (
              mod.constructor.name === 'AutoPublicPathRuntimeModule' &&
              kind === 'worker'
            ) {
              // A classic worker uses its script URL without introducing import.meta syntax.
              const workerSource =
                () => `var script = self.__workerEntryURL || self.location.href;
            __webpack_require__.p = new URL(".", script).href;`;
              if (rspackMode) mod.generate = workerSource;
              else mod.source.source = workerSource();
            } else if (mod.constructor.name === 'AutoPublicPathRuntimeModule') {
              // tramvai patches ordinary JS, while Fluent UI and Hasura wrap generate.
              const source = mod.source.source
                .toString()
                .replace(
                  'document.currentScript.src;',
                  'document.currentScript.src.replace("index.js", "worker/index.js");',
                );
              mod.source.source = Buffer.from(source);
              const original = mod.generate.bind(mod);
              const generate = () =>
                original() + '\n__webpack_require__.p += "generated/";';
              if (rspackMode) mod.generate = generate;
              else mod.source.source = generate();
            } else if (mod.name === 'global') {
              const generate = () =>
                '__webpack_require__.g = globalThis; __webpack_require__.globalHookRan = true;';
              if (rspackMode) mod.generate = generate;
              else mod.source.source = generate();
            } else if (mod.name === 'extension paths') {
              // tramvai's later JS generator sees the condition map on the attached graph.
              const condition = Reflect.get(
                mod,
                'chunkGraph',
              ).getChunkConditionMap(chunk, hasEntry);
              mod.source.source = `${mod.generate()}\n__webpack_require__.webExtRt.condition = ${JSON.stringify(condition[chunk.id!])};`;
            }
          });
        });
      }),
    ],
  }),
);
