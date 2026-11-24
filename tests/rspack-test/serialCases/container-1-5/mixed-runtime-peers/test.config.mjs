import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';

export default {
  findBundle() { return []; },
  afterExecute(configs) {
    for (const { output } of configs) {
      const id = output.uniqueName;
      if (!id.endsWith('-mixed')) continue;
      const run = (variant) => JSON.parse(execFileSync(process.execPath, [
        '-e', `
          const path = require('node:path');
          const [root, id] = process.argv.slice(1);
          const warnings = [];
          console.warn = (...args) => warnings.push(args.join(' '));
          globalThis.__mfPeerLoad = async (name) => require(path.join(root, name, 'remoteEntry.js'));
          require(path.join(root, id, 'main.js'))().then((result) => {
            console.log(JSON.stringify({ ...result, initWarnings: warnings.filter((warning) => warning.includes('Initialization of sharing external failed')) }));
          }, (error) => { console.error(error); process.exitCode = 1; });
        `,
        output.path, variant,
      ], { encoding: 'utf8', timeout: 10000 }));
      const reference = run(id.replace(/-mixed$/, '-reference'));
      assert.equal(reference.host, id === 'transitive-mixed' ? 'last' : 'host', id);
      assert.equal(reference.remote, reference.host, id);
      assert.equal(reference.sameObject, true, id);
      assert.deepEqual(reference.initWarnings, [], id);
      assert.deepEqual(run(id), reference, id);
    }
  },
};
