it('calls the host error handler when a bundled factory throws', () => {
  const errors = [];
  globalThis.ErrorUtils = { reportFatalError: error => errors.push(error.message) };
  try {
    require('./broken.cjs');
    expect(errors).toEqual(['module failed']);
  } finally {
    delete globalThis.ErrorUtils;
  }
});
