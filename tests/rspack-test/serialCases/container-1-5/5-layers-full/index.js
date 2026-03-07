it('should load App with React', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toBe(
      'App rendered with React version: [This is react 0.1.2] with layer [This is layered react] ComponentA rendered with React version: [This is react 0.1.2] with layer [This is layered react]',
    );
  });
});
