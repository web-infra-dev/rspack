it('should have default export for json module', async () => {
  const { default: json } = await import(/* webpackIgnore: true */'./bundle.mjs')
  expect(json.foo).toBe(42)
  const { default: cjs } = await import(/* webpackIgnore: true */'./cjsBundle.mjs')
  expect(cjs.value).toBe(42)
})