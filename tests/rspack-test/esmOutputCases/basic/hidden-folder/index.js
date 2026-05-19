it("should render hidden folder", async () => {
  const {value} = await import('./.hidden/index')

  expect(value()).toBe(42)
})