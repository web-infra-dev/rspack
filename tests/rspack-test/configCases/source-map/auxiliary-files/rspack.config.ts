import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  devtool: 'source-map',
  plugins: [
    definePlugin((compiler) => {
      const { Compilation } = compiler.rspack;
      compiler.hooks.thisCompilation.tap('test case', (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'test case',
            stage: Compilation.PROCESS_ASSETS_STAGE_REPORT,
          },
          () => {
            const [chunk] = compilation.chunks;
            const filesView = chunk.files;
            const auxiliaryView = chunk.auxiliaryFiles;
            const runtimeView = chunk.runtime;
            const files = [...filesView];
            const auxiliaryFiles = [...auxiliaryView];
            const fileIterator = filesView.values();
            const auxiliaryIterator = auxiliaryView.values();
            expect(filesView.size).toBe(1);
            expect(filesView.has('bundle0.js')).toBe(true);
            expect(auxiliaryView.size).toBe(1);
            expect(auxiliaryView.has('bundle0.js.map')).toBe(true);
            expect(runtimeView.size).toBe(1);
            expect(runtimeView.has('main')).toBe(true);
            expect(runtimeView.has('missing-runtime')).toBe(false);
            expect(chunk.runtime).toBe(runtimeView);
            for (const view of [filesView, auxiliaryView, runtimeView]) {
              expect(Reflect.apply(view.has, view, [null])).toBe(false);
              expect(Reflect.apply(view.has, view, [123])).toBe(false);
              expect(Reflect.apply(view.has, view, [])).toBe(false);
            }
            expect(files).toEqual(['bundle0.js']);
            expect(auxiliaryFiles).toEqual(['bundle0.js.map']);
            expect([...runtimeView]).toEqual(['main']);

            compilation.renameAsset('bundle0.js', 'renamed.js');
            compilation.renameAsset('bundle0.js.map', 'renamed.js.map');
            expect(chunk.files).toBe(filesView);
            expect(chunk.auxiliaryFiles).toBe(auxiliaryView);
            expect(filesView.size).toBe(1);
            expect(filesView.has('bundle0.js')).toBe(false);
            expect(filesView.has('renamed.js')).toBe(true);
            expect(auxiliaryView.size).toBe(1);
            expect(auxiliaryView.has('bundle0.js.map')).toBe(false);
            expect(auxiliaryView.has('renamed.js.map')).toBe(true);
            expect([...filesView]).toEqual(['renamed.js']);
            expect([...auxiliaryView]).toEqual(['renamed.js.map']);
            expect([...fileIterator]).toEqual(['bundle0.js']);
            expect([...auxiliaryIterator]).toEqual(['bundle0.js.map']);
            expect(files).toEqual(['bundle0.js']);
            expect(auxiliaryFiles).toEqual(['bundle0.js.map']);

            compilation.renameAsset('renamed.js', 'bundle0.js');
            compilation.renameAsset('renamed.js.map', 'bundle0.js.map');
          },
        );
      });
    }),
  ],
});
