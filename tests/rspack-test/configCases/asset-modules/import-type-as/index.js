import plain from './plain.js';
import plainSource from './plain.js' with { type: 'text' };
import plainBytes from './plain.js' with { type: 'bytes' };
import cssUrl from './example.css';
import cssText from './example.css' with { type: 'text' };
import kept from './intentional.js' with { type: 'text' };
import rewritten from './plain.js?rewritten' with { type: 'text' };
import jsonData from './plain.js?json' with { type: 'json' };

const ORIGINAL_PLAIN = 'export default "__MARK__";';
const ORIGINAL_CSS = 'body { color: red; }';

it('skips original extension rules for text and bytes imports', async () => {
  const decoder = new TextDecoder('utf-8');

  expect(plain).toBe('compiled');
  expect(cssUrl).toMatch(/\.css$/);

  expect(plainSource).toBe(ORIGINAL_PLAIN);
  expect(
    (await import('./plain.js', { with: { type: 'text' } })).default,
  ).toBe(ORIGINAL_PLAIN);
  expect(
    (await import('./plain.js?other', { with: { type: 'text' } })).default,
  ).toBe(ORIGINAL_PLAIN);

  expect(plainBytes.constructor.name).toBe('Uint8Array');
  expect(decoder.decode(plainBytes)).toBe(ORIGINAL_PLAIN);
  expect(
    decoder.decode(
      (await import('./plain.js', { with: { type: 'bytes' } })).default,
    ),
  ).toBe(ORIGINAL_PLAIN);
  expect(
    decoder.decode(
      (await import('./plain.js?other', { with: { type: 'bytes' } })).default,
    ),
  ).toBe(ORIGINAL_PLAIN);

  expect(cssText).toBe(ORIGINAL_CSS);
  expect(
    (await import('./example.css', { with: { type: 'text' } })).default,
  ).toBe(ORIGINAL_CSS);
});

it('keeps transforms that match import attributes or rewritten suffixes and leaves json unchanged', () => {
  expect(kept).toBe('kept');
  expect(rewritten).toBe('export default "rewritten";');
  expect(jsonData).toEqual({ unchangedJsonRule: true });
});
