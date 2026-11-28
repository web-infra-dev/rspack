import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
const issuerSnapshots: (string | null)[] = [];

export default defineConfig({
  mode: 'development',
  devtool: false,
  entry: './index.js',
  cache: { type: 'persistent' },
  incremental: {
    silent: true,
    buildModuleGraph: true,
    finishModules: false,
    optimizeDependencies: false,
    buildChunkGraph: false,
    optimizeChunkModules: false,
    moduleIds: false,
    chunkIds: false,
    modulesHashes: false,
    modulesCodegen: false,
    modulesRuntimeRequirements: false,
    chunksRuntimeRequirements: false,
    chunksHashes: false,
    chunkAsset: false,
    emitAssets: false,
  },
  output: { filename: 'main.js' },
  plugins: [
    definePlugin({
      name: 'CaptureSharedIssuer',
      apply(compiler) {
        this.compiler = compiler;
        this.issuerSnapshots = issuerSnapshots;
        this.modifiedFiles = undefined;
        compiler.hooks.run.tap('CaptureSharedIssuer', () => {
          // Apply the case's edits after HotUpdatePlugin's beforeRun hook.
          if (this.modifiedFiles) compiler.modifiedFiles = this.modifiedFiles;
        });
        compiler.hooks.done.tap('CaptureSharedIssuer', (stats) => {
          expect(stats.hasErrors()).toBe(false);
          const shared = [...stats.compilation.modules].find(
            (module) =>
              (module as { resource?: string }).resource?.replace(
                /^.*[\\/]/,
                '',
              ) === 'shared.js',
          );
          expect(shared).toBeDefined();
          const issuer = stats.compilation.moduleGraph.getIssuer(shared!);
          issuerSnapshots.push(
            (issuer as { resource?: string } | null)?.resource ?? null,
          );
          expect(issuerSnapshots[buildIndex]).toMatch(/[\\/][ab]\.js$/);
          if (buildIndex > 0) {
            expect(issuerSnapshots[1]).not.toBe(issuerSnapshots[0]);
            expect(issuerSnapshots[buildIndex]).toBe(issuerSnapshots[1]);
          }
          const modules = stats.toJson({ all: false, cacheInfo: true })
            .cacheInfo!['|Compilation/modules'];
          // Unrelated edits must only store the changed module, not shared.js
          // again because of an issuer update left over from the previous run.
          expect(modules.stores).toBe([5, 1, 1, 1, 0][buildIndex]);
          if (buildIndex === 4) {
            expect(issuerSnapshots).toHaveLength(5);
            expect(modules.reused).toBe(4);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
