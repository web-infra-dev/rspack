import assert from "node:assert/strict";

export default async function () {
  assert.equal(this.channelCompiler, this._compiler.name);
  await new Promise((resolve) => setImmediate(resolve));
  return `module.exports = ${JSON.stringify(this._compiler.name)};`;
}
