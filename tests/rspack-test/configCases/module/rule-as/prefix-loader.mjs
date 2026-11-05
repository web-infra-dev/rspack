import assert from 'node:assert/strict';

export default function (source) {
  assert.match(this.resourcePath, /(?:example|literal)\.source\.js$/);
  assert.equal(
    this.resource,
    this.resourcePath + this.resourceQuery + this.resourceFragment,
  );
  if (this._module.matchResource) {
    assert.equal(this.resourceQuery, '?original');
    assert.equal(this.resourceFragment, '#real');
  }
  return `${this.getOptions().prefix}${source}`;
}
