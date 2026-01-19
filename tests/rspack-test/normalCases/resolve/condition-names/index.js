import a from 'a';
import b from 'b';

it('should use rspack as condition names', async () => {
  expect(a).toBe('a');
})

it('should fallback to webpack as condition names', async () => {
  expect(b).toBe('b');
})