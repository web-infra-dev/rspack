it('should refresh lazy fields across Rust and JavaScript loader yields', () => {
  expect(require('./fixture')).toBe(42);
});
