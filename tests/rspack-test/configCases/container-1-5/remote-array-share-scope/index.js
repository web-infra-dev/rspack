it('should reuse specific share scope', async () => {
  const App = await import('./App.js');
  expect(App.default()).toBe(`UiLib1: This is @scope-sc/ui-lib 0.1.3
  UiLib2: This is @scope-sc/ui-lib2 0.1.4
  UiLib3: This is @scope-sc/ui-lib3 0.1.5
  `);
  const Expose = await import('remote-alias/Expose');
  // TheUiLib3 should not shared
  expect(Expose.default()).toBe(`UiLib1: This is @scope-sc/ui-lib 0.1.3
  UiLib2: This is @scope-sc/ui-lib2 0.1.4
  UiLib3: This is @scope-sc/ui-lib3 0.1.2
  `);

});
