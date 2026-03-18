it('should load the component from container', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toBe(
      'App rendered with [This is react 0.1.2] No Layer (1-layers-full), ComponentALayers This is react 0.1.2 rendered with [This is layered react (1-layers-full)], ComponentA rendered with [This is react 0.1.2]No Layer (1-layers-full), [ComponentA rendered with [This is react 0.1.2]No Layer (1-layers-full)] and [ComponentALayers This is react 0.1.2 rendered with [This is layered react (1-layers-full)]]',
    );
  });
});
