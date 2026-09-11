import './shadowed';
import './duplicate.cjs';

it('keeps shadowed callees and merged declarations conservative', () => {
  expect(globalThis.__semantic_shadowed_calls__).toEqual(['parameter', 'block']);
  expect(globalThis.__semantic_duplicate_calls__).toBe(1);
  delete globalThis.__semantic_shadowed_calls__;
  delete globalThis.__semantic_duplicate_calls__;
});
