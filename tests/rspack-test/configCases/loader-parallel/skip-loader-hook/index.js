it('skips the hook for worker pitch and normal stages in a mixed chain', () => {
  expect(require('./value')).toBe('worker:main');
});
