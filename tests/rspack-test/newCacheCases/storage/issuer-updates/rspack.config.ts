import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
const issuerSnapshots: (string | null)[] = [];

export default defineConfig({
  mode: 'development',
  devtool: false,
  entry: './index.js',
  cache: { type: 'persistent' },
  output: { filename: 'main.js' },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('CaptureSharedIssuer', (stats) => {
          const shared = [...stats.compilation.modules].find(
            (module) =>
              (module as { resource?: string }).resource?.replace(
                /^.*[\\/]/,
                '',
              ) === 'shared.js',
          );
          const issuer =
            shared && stats.compilation.moduleGraph.getIssuer(shared);
          issuerSnapshots.push(
            (issuer as { resource?: string } | null)?.resource ?? null,
          );
          if (buildIndex === 3) {
            expect(issuerSnapshots).toHaveLength(4);
            expect(issuerSnapshots[1]).not.toBe(issuerSnapshots[0]);
            expect(issuerSnapshots.slice(1)).toEqual([
              issuerSnapshots[1],
              issuerSnapshots[1],
              issuerSnapshots[1],
            ]);
          }
          buildIndex++;
        });
      },
    }),
  ],
});
