it('should load App with React', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toBe(
      'App (no layer) rendered with React version: [This is react 0.1.2] with non-layered React value: [No Layer] and imported: ComponentA (in react-layer) rendered with React version: [This is react 0.1.2] with layered React value: [react-layer]',
    );
  });
});
