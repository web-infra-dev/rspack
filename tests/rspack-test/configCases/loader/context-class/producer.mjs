import assert from 'node:assert/strict';
import path from 'node:path';

export function pitch() {
  this.data.phase = 'pitched';
  this.addDependency(path.join(this.context, 'pitch.txt'));
}

export default function (source) {
  assert.equal(this.data.phase, 'pitched');
  assert.ok(this.getDependencies().some(file => file.endsWith('/pitch.txt')));
  const callback = this.async();
  setTimeout(() => {
    this.addDependency(path.join(this.context, 'normal.txt'));
    const additionalData = { value: () => 42 };
    additionalData.self = additionalData;
    callback(null, source, {
      version: 3,
      sources: [this.resourcePath],
      sourcesContent: [source],
      names: [],
      mappings: 'AAAA',
    }, additionalData);
  }, 0);
}
