const { execFileSync } = require('node:child_process');
const path = require('node:path');
it('reloads a Node entry while callers retain its original CommonJS namespace', () => {
  execFileSync(process.execPath, ['--eval', `
    const assert = require('node:assert/strict');
    const file = ${JSON.stringify(path.join(__dirname, 'entry.js'))};
    const entry = require(file);
    assert.equal(entry.supportsReload(), ${__TEST_ENABLED__});
    if (!${__TEST_ENABLED__}) {
      assert.equal(entry.render(), 'generation:1');
      process.exit(0);
    }
    const firstState = entry.state;
    const firstRender = entry.render;
    assert.equal(entry.render(), 'generation:1');
    for (let generation = 2; generation <= 20; generation++) {
      entry.reload();
      assert.strictEqual(require(file), entry);
      assert.strictEqual(require.cache[file].exports, entry);
      assert.equal(entry.generation, generation);
      assert.equal(entry.render(), 'generation:' + generation);
      assert.equal(entry.state.generation, generation);
      assert.notStrictEqual(entry.state, firstState);
    }
    // Values already captured by callers are not rewritten by namespace getters.
    assert.equal(firstRender(), 'generation:1');
    assert.equal(firstState.generation, 1);
  `], { stdio: 'pipe' });
});
