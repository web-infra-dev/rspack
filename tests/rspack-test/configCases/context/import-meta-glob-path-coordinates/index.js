import {
  bracedExclusion,
  outsideGlob,
  parentExclusion,
  parentGlob,
} from './a/b/globs.js';

const equivalentRoot = import.meta.glob('./outside/*.js', {
  eager: true,
  import: 'default',
  query: '?equivalent-root',
});

it('matches parent patterns in coordinates relative to the importer', () => {
  expect(parentGlob).toEqual({ '../sibling/other.mjs': 'sibling' });
});

it('keeps importer files when a negative glob targets the parent directory', () => {
  expect(parentExclusion).toEqual({ './local.mjs': 'local' });
});

it('applies a negative brace branch outside the shared literal prefix', () => {
  expect(outsideGlob).toEqual({ '../../outside/x.js': 'outside' });
  expect(bracedExclusion).toEqual({});
});

it('matches files when afterResolve returns an equivalent directory', () => {
  expect(equivalentRoot).toEqual({ './outside/x.js': 'outside' });
});
