it('other.js should load the component from container', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toBe(
      'App rendered with [This is react 1.2.3] and [ComponentA rendered with [This is react 1.2.3]]',
    );
  });
});
