function missingUrl() {
  return new URL('./missing-target.txt', import.meta.url);
}

it('should retain normal missing-module diagnostics after a failed type probe', () => {
  expect(missingUrl).toThrow(/Cannot find module/);
});
