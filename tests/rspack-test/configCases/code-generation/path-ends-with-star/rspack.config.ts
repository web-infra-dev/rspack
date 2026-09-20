import { defineConfig, definePlugin } from '@rspack/cli';
import fs2 from 'node:fs';
import path2 from 'node:path';

const isWindows = process.platform === 'win32';

const entry = `it("should generate valid code", async () => {${
  isWindows
    ? `expect("skip windows").toBe("skip windows");`
    : `const { staticA, dynamicA } = await import("./entry.mjs"); expect(staticA.a).toBe(1); expect(dynamicA.a).toBe(1);`
}});`;

export default defineConfig({
  entry: `data:text/javascript,${entry}`,
  plugins: [
    definePlugin(function skipWindows(compiler) {
      // windows' path can't include *
      if (!isWindows) {
        const fs = fs2;
        const path = path2;
        // `isolateSource` (test.config.mjs) makes __dirname a per-suite copy, so
        // creating/removing `star*` here can't race the parallel RuntimeMode suite.
        const dir = path.resolve(import.meta.dirname, 'star*');
        if (!fs.existsSync(dir)) {
          fs.mkdirSync(dir);
        }
        fs.writeFileSync(path.resolve(dir, 'a.js'), 'export const a = 1;');
        // cleanup
        compiler.hooks.done.tap('skipWindows', () => {
          fs.rmSync(dir, { recursive: true, force: true });
        });
      }
    }),
  ],
});
