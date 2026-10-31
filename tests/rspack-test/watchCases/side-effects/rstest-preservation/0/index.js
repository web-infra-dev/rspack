import { trace } from 'preservation-state';
import { value } from './trigger';

it('updates protection without rebuilding unrelated importers', async () => {
  expect(value).toBe(WATCH_STEP);
  trace.length = 0;
  expect((await import('importer-a')).value).toBe('a');
  expect((await import('importer-b')).value).toBe('b');
  expect((await import('untouched')).value).toBe('untouched');
  expect(trace).toEqual([WATCH_STEP === '1' ? 'b' : 'a']);
});
