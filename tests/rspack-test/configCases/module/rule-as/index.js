import source from './example.source.js';
import sourceWithQuery from './example.source.js?inline#fragment';
import ordinary from './ordinary.js';

it('matches later and nested rules using the as filename', () => {
  expect(source).toContain('original source');
  expect(source).toContain('nested:');
  expect(source).toContain('next:');
  expect(source).toContain('async:');
  expect(source).not.toContain('wrong:');
  expect(sourceWithQuery).toContain('original source');
  expect(sourceWithQuery).toContain('async:');
  expect(ordinary).toBe('ordinary');
});
