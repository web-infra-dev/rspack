it('should load the component from container', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toContain('App rendered');
    return import('./upgrade-react').then(({ default: upgrade }) => {
      upgrade();
      const rendered = App();
      expect(rendered).toContain('App rendered');
    });
  });
});
