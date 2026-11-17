it('should preserve additionalData identity across JS, worker and builtin boundaries', () => {
  expect(require('./resource')).toEqual({ count: 2, buffer: 'native-handle', value: 42 });
});
