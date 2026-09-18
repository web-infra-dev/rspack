import assert from "node:assert/strict";

export default async function () {
  const { name, fail } = this.getOptions();
  assert.equal(this._compiler.name, name);
  assert.equal(this.channelCompiler, name);
  const nested = await this.importModule(
    `!!${import.meta.dirname}/nested.mjs!${this.resourcePath}?nested`,
  );
  assert.equal(nested, name);
  if (fail) throw new Error(`intentional channel error: ${name}`);
  return `module.exports = ${JSON.stringify(nested)};`;
}
