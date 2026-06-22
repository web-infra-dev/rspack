const fs = require('node:fs');
const path = require('node:path');

// `ctx/__ignored__/seed.js` is a bundled member of the watched `require.context`
// (see 0/index.js), so a normal edit would show up in `compiler.modifiedFiles`
// and drive a rebuild — unless the watcher filters its `__ignored__` subtree via
// `watchOptions.ignored`. Step 1 edits both `ctx/trigger.js` (not ignored) and
// `ctx/__ignored__/seed.js` (ignored) at once; the rebuild's `modifiedFiles`
// must contain the former and exclude the latter.
class IgnoredContextProbe {
  apply(compiler) {
    const probeFile = path.join(compiler.options.output.path, 'probe.json');
    compiler.hooks.done.tap('IgnoredContextProbe', () => {
      fs.mkdirSync(path.dirname(probeFile), { recursive: true });
      fs.writeFileSync(
        probeFile,
        JSON.stringify({
          modifiedFiles: Array.from(compiler.modifiedFiles || []),
        }),
      );
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  watchOptions: {
    aggregateTimeout: 200,
    ignored: ['**/__ignored__'],
  },
  plugins: [new IgnoredContextProbe()],
};
