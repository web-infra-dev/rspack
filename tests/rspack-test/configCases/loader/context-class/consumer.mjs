import assert from 'node:assert/strict';

export function pitch() {
  this.data.phase = 'pitched';
  assert.equal(this.getDependencies().some(file => file.endsWith('/pitch.txt')), false);
}

export default function (source, sourceMap, additionalData) {
  assert.equal(this.data.phase, 'pitched');
  assert.ok(this.getDependencies().some(file => file.endsWith('/pitch.txt')));
  assert.ok(this.getDependencies().some(file => file.endsWith('/normal.txt')));
  assert.equal(sourceMap.version, 3);
  assert.equal(sourceMap.sourcesContent[0], source);
  assert.equal(additionalData.self, additionalData);
  assert.equal(additionalData.value(), 42);
  this.__internal__setParseMeta('contextClass', 'updated');
  this.callback(null, `module.exports = ${additionalData.value()};`, sourceMap);
}
