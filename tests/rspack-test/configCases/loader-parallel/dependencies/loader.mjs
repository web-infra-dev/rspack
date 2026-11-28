import assert from 'node:assert/strict';
import { join } from 'node:path';

/** @type {import('@rspack/core').LoaderDefinition} */
export default function (source) {
  assert.equal(this.parallel, true);
  const file = join(this.context, 'test.config.mjs');
  const missing = join(this.context, 'missing.js');
  this.dependency(file);
  this.addContextDependency(this.context);
  this.addMissingDependency(missing);
  this.addBuildDependency(this.resourcePath);
  assert.ok(this.getDependencies().includes(file));
  assert.ok(this.getContextDependencies().includes(this.context));
  assert.ok(this.getMissingDependencies().includes(missing));

  this.clearDependencies();
  this.addDependency(this.resourcePath);
  assert.deepEqual(this.getDependencies(), [this.resourcePath]);
  assert.deepEqual(this.getContextDependencies(), []);
  assert.deepEqual(this.getMissingDependencies(), []);

  return source;
}
