const targetUrl = new URL('./target', import.meta.url);

it('should use the asset URL when an unchanged issuer resolves to an asset after rebuild', () => {
  if (WATCH_STEP === '0' || WATCH_STEP === '2') {
    expect(targetUrl.href).toMatch(/\/assets\/url-[^/]+\.js$/);
  } else {
    expect(targetUrl.href).toMatch(/\/assets\/target\.txt$/);
  }
  expect(globalThis.URL_TARGET_TYPE_CHANGE_EXECUTED).toBeUndefined();
});
