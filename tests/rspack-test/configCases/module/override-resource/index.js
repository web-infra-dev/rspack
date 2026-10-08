import source from './example.source.js';
import sourceWithQuery from './example.source.js?inline#fragment';
import overridden from './example.source.js?override#original';
import removed from './single.js?remove#original';
import hidden from './.hidden';
import hiddenWithExtension from './.hidden.local';
import trailing from './trailing.';
import extensionless from './README';
import unicode from './unicode.js';
import ordinary from './ordinary.js';

it('matches later and nested rules using the overridden extension', () => {
  const data = JSON.parse(source);
  expect(data.source).toContain('original source');
  expect(data.source).toContain('nested:');
  expect(data.source).toContain('next:');
  expect(data.source).toContain('path:');
  expect(data.source).not.toContain('wrong:');
  expect(data.filename).toBe('example.source.js');
  expect(data.ext).toBe('.js');
  expect(ordinary).toBe('ordinary');
});

it('inherits query and fragment when they are omitted', () => {
  const data = JSON.parse(sourceWithQuery);
  expect(data.source).toContain('preserved:');
  expect(data.query).toBe('?inline');
  expect(data.fragment).toBe('#fragment');
});

it('overrides and clears query and fragment for subsequent rules', () => {
  const data = JSON.parse(overridden);
  expect(data.source).toContain('nested-query:');
  expect(data.source).toContain('changed:');
  expect(data.source).toContain('cleared:');
  expect(data.query).toBe('?override');
  expect(data.fragment).toBe('#original');
  expect(data.ext).toBe('.js');
});

it('removes the extension with an empty override', () => {
  const data = JSON.parse(removed);
  expect(data.source).toContain('removed:');
  expect(data.filename).toBe('single.js');
  expect(data.query).toBe('?remove');
});

it('exposes extensions with Node.js extname semantics', () => {
  for (const [value, ext] of [
    [hidden, ''],
    [hiddenWithExtension, '.local'],
    [trailing, '.'],
    [extensionless, ''],
    [unicode, '.js'],
  ]) {
    const data = JSON.parse(value);
    expect(data.ext).toBe(ext);
    expect(data.source).toContain('nested:');
    expect(data.source).toContain('next:');
  }
});
