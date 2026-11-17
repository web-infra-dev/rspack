const assert = require("node:assert/strict");
const path = require("node:path");
const { isMainThread } = require("node:worker_threads");
const delay = () => new Promise(resolve => setTimeout(resolve, 20));
let activeLeaves = 0;

module.exports = async function (source) {
  assert.equal(isMainThread, !this.parallel);
  assert.equal(this.sourceMap, true);
  const name = path.basename(this.resourcePath);
  if (name === "receive-root.cjs") {
    // Concurrent calls each allow recursive work; the child itself imports a leaf.
    const [child, leaf] = await Promise.all([
      this.importModule("./receive-child.cjs"),
      new Promise((resolve, reject) => this.importModule(
        "./receive-leaf.cjs?callback", {},
        (error, value) => error ? reject(error) : resolve(value),
      )),
    ]);
    await assert.rejects(this.importModule("./receive-fail.cjs"), /receive failure/);
    assert.equal(await this.importModule("./receive-leaf.cjs?after-error"), 7);
    // The import scopes have ended. Ordinary async loaders must remain serial.
    return `require("./receive-other.cjs?1"); require("./receive-other.cjs?2");
      require("./receive-other.cjs?3"); module.exports = ${child + leaf};`;
  }
  if (name === "receive-child.cjs") {
    const leaf = await this.importModule("./receive-leaf.cjs?nested");
    // These build tasks are queued after the child loader finishes. An import
    // scope must keep receiving, rather than consuming just one task.
    return Array.from({ length: 5 }, (_, i) =>
      `require("./receive-leaf.cjs?graph${i}");`
    ).join("\n") + `module.exports = ${leaf * 5};`;
  }
  if (name === "receive-fail.cjs") throw new Error("receive failure");
  if (name === "receive-other.cjs" && this.parallel) {
    assert.equal(++activeLeaves, 1, "ordinary tasks must wait for async completion");
    try { await delay(); } finally { activeLeaves--; }
  }
  return source;
};
