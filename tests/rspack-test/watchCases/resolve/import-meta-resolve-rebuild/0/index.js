const targetUrl = new URL(import.meta.resolve('./target'));

it('should keep URL entries unique and update their type across rebuilds', () => {
  if (WATCH_STEP === '2' || WATCH_STEP === '4') {
    expect(targetUrl.href).toMatch(/\/assets\/target\.txt$/);
  } else {
    expect(targetUrl.href).toMatch(/\/assets\/url-[^/]+\.js$/);
  }
  expect(globalThis.URL_ENTRY_REBUILD_TARGET_EXECUTED).toBeUndefined();
});
