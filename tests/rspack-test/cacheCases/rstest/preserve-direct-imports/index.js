import { trace } from './state';
import { value as generation } from './generation';

it('updates cached importers when protection changes', async () => {
  expect(generation).toBe(COMPILER_INDEX);
  trace.length = 0;
  expect((await import('./a')).value).toBe('a');
  expect((await import('./b')).value).toBe('b');
  expect((await import('./untouched')).value).toBe('untouched');
  expect(trace).toEqual([COMPILER_INDEX % 2 === 0 ? 'a' : 'b']);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
