const assert = require('assert');
const path = require('path');

module.exports = function (content) {
  if (path.basename(this.resourcePath) !== 'lib.ts') {
    return content;
  }

  const info = this._module.buildInfo.collectedTypeScriptInfo;

  assert.ok(info, 'collectedTypeScriptInfo should be exposed to JS loaders');
  assert.ok(info.typeExports instanceof Set, 'typeExports should be a Set');
  assert.ok(info.exports instanceof Set, 'exports should be a Set');
  assert.ok(info.importedModules instanceof Set, 'importedModules should be a Set');
  assert.deepStrictEqual(
    Array.from(info.typeExports).sort(),
    ['Bar', 'Baz', 'Foo', 'Inline', 'ReExportedType', 'default'],
  );
  assert.deepStrictEqual(
    Array.from(info.exports).sort(),
    [
      'Bar',
      'Baz',
      'Foo',
      'Inline',
      'ReExportedType',
      'default',
      'loadDynamic',
      'namespace',
      'renamedValue',
      'value',
    ],
  );
  assert.deepStrictEqual(
    Array.from(info.importedModules).sort(),
    ['./dep', './dynamic', './inline-type', './namespace', './star', './types'],
  );
  assert.ok(!info.typeExports.has('value'), 'value exports should not be included');

  return content;
};
