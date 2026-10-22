import value, { named as renamed, bump } from './values.js';
import * as namespace from './values.js';
import { createRequire as makeRequire } from 'node:module';
import './effect.js';
import snapshot, { current, change } from './default-value.js';
import exportedNamespace from './default-namespace.js';
import anonymous from './default-function.js';
import Named from './default-class.js';

it('preserves default export snapshots and namespace live bindings', () => {
  expect(snapshot).toBe(3);
  change();
  expect(snapshot).toBe(3);
  expect(current).toBe(5);
  expect(exportedNamespace.current).toBe(5);
  expect(anonymous()).toBe(17);
  expect(new Named().value()).toBe(19);
});

const localRequire = makeRequire(import.meta.url);

it('preserves import tags, live bindings and local shadowing', () => {
  expect(value).toBe(7);
  expect(renamed).toBe(1);
  bump();
  expect(renamed).toBe(2);
  expect(namespace.named).toBe(2);
  expect(((renamed) => renamed)(11)).toBe(11);
  expect(globalThis.__importSemanticEffect).toBe(true);
  expect(localRequire('./cjs.js')).toBe(13);
  delete globalThis.__importSemanticEffect;
});
