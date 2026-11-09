it('serializes loader options and additionalData across main, worker and builtin boundaries', () => {
  expect(require('./resource')).toEqual({
    count: 2,
    value: 9,
    buffer: 'additional data',
  });
});
