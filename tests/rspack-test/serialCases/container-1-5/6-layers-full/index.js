it('should load App with React and remote component', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toBe(
      'App rendered with React version: [This is react 0.1.2]\nand remote component: [ComponentA rendered with React version: [This is react 0.1.2] with layer [This is layered react]]\n and local component: [ComponentA with React: This is react 0.1.2 layered with This is layered react]',
    );
  });
});
