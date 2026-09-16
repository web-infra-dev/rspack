export const generation = 1;
const id = module.id;
it('keeps the initially published Node namespace live after entry replacement', () => {
  const initial = globalThis.__TEST_EXPORTED__;
  expect(initial.generation).toBe(1);
  expect(initial.__esModule).toBe(true);
  expect(Object.prototype.toString.call(initial)).toBe('[object Module]');
  expect(Object.keys(initial)).toEqual(['generation']);
  expect(__TEST_RUNTIME__.updateEntryExports(id, { generation: 2 })).toBe(true);
  expect(initial.generation).toBe(2);
  expect(__TEST_RUNTIME__.updateEntryExports(id, { generation: 3 })).toBe(true);
  expect(initial.generation).toBe(3);
  delete globalThis.__TEST_EXPORTED__;
});
