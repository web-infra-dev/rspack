it('should load App with React and remote component', async () => {
  const App = (await import('./App')).default;
  const upgrade = (await import('./upgrade-react')).default;
  upgrade();
  const rendered = App();
  expect(rendered).toBe(
    'App rendered with React version: [This is react 1.2.3]\nand remote component: [ComponentA rendered with React version: [This is react 1.2.3]]',
  );
});
