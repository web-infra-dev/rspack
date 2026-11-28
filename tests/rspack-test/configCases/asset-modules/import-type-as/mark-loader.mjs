import assert from 'node:assert/strict';

export default function (source) {
  assert.equal(
    this.resource,
    this.resourcePath + this.resourceQuery + this.resourceFragment,
  );
  assert.doesNotMatch(this.resourcePath, /\.(?:text|bytes)$/);
  const { raw, value = 'compiled' } = this.getOptions();
  if (raw !== undefined) {
    return raw;
  }
  return source.replaceAll('__MARK__', value);
}
