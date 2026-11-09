it('runs pure pitch and normal loaders in a worker and writes an asset', () => {
  expect(require('./value')).toBe('value:pure:worker:main');
});
