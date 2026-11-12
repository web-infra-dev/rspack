it('should load CSS from a named asynchronous import', async () => {
  await import(/* webpackChunkName: "controls" */ './controls.css');
});
