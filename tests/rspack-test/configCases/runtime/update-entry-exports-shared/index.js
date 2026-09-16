const id = module.id;
it('reports unsupported ownership for an entry outside the runtime chunk', () => {
  expect(__TEST_RUNTIME__.updateEntryExports(id, {})).toBe(false);
});
