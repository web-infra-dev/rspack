import shared from './shared.cjs';

it('should not execute another entry while loading a shared chunk', async () => {
  expect(other).toBe(42);

  const runsBefore = globalThis.__rspack_test_entry_cycle_runs;
  const main = await import(/* webpackIgnore: true */ MAIN_BUNDLE);

  expect(main.a).toBe(43);
  const c = await main.loadC();
  expect(c.default).toBe(2);
  // The other entry's modules must run exactly once, and only because they were
  // imported here: loading this bundle must not evaluate the shared chunk's
  // registration by importing a runtime chunk that also carries modules.
  expect(globalThis.__rspack_test_entry_cycle_runs).toBe((runsBefore || 0) + 1);
});

export const other = shared.value;
