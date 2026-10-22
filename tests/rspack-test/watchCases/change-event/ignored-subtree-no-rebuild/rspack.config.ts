import { defineConfig } from '@rspack/cli';
import { type Compiler } from '@rspack/core';
import fs from 'node:fs';
import path from 'node:path';
// `aggregateTimeout` is far smaller than `SETTLE_MS`: a rebuild wrongly
// triggered by the ignored change would land well before `trigger.js` is
// touched, bumping the build count the probe records.
const SETTLE_MS = 1500;

class IgnoredRebuildProbe {
  builds = 0;

  apply(compiler: Compiler) {
    const ignoredFile = path.join(compiler.context, '__ignored__', 'noise.js');
    const triggerFile = path.join(compiler.context, 'trigger.js');
    const probeFile = path.join(compiler.options.output.path!, 'probe.json');

    compiler.hooks.done.tap(IgnoredRebuildProbe.name, () => {
      this.builds += 1;
      fs.mkdirSync(path.dirname(probeFile), { recursive: true });
      fs.writeFileSync(probeFile, JSON.stringify({ builds: this.builds }));
      if (this.builds === 1) {
        fs.writeFileSync(ignoredFile, `// noise ${Date.now()}`);
        setTimeout(() => {
          fs.writeFileSync(triggerFile, 'module.exports = "changed";');
        }, SETTLE_MS);
      }
    });
  }
}

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  watchOptions: {
    aggregateTimeout: 200,
    ignored: ['**/__ignored__'],
  },
  plugins: [new IgnoredRebuildProbe()],
});
