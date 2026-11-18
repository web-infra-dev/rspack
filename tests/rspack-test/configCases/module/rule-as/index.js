import source from './example.source.js';
import sourceWithQuery from './example.source.js?inline#fragment';
import sourceWithMatchResource from './virtual.source.js?inline#fragment!=!./example.source.js?original#real';
import literal from './literal.source.js?literal\u200b#value#fragment!=!./literal.source.js?original#real';
import ordinary from './ordinary.js';

it('applies templates sequentially to later and nested rules without restarting', () => {
  expect(source).toBe('nested:next:child:final:selected:original source\n');
});

it('preserves query and fragment when replacing the matching resource', () => {
  expect(sourceWithQuery).toBe(
    'nested:next:child:final:selected:query:original source\n',
  );
});

it('expands the current inline match resource path', () => {
  expect(sourceWithMatchResource).toBe(sourceWithQuery);
});

it('preserves literal path delimiters, query, and fragment during matching', () => {
  expect(literal).toBe('choice:literal:original source\n');
});

it('does not change unrelated modules', () => {
  expect(ordinary).toBe('ordinary');
});
