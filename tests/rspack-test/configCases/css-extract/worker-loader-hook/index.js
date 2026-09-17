it('extracts CSS after initializing the loader context', () => {
  expect(() => require('./style.css')).not.toThrow();
});
