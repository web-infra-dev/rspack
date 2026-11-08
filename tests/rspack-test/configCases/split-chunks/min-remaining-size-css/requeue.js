it('should retry a smaller candidate without losing its JavaScript modules', async () => {
  const module = await import(/* webpackChunkName: "controls" */ './mixed');
  expect(module.default).toBe('shared');
});
