it("should import context module", async () => {
  const n = "a"
  const { default: a1 } = await import(`./sub/${n}`);
  expect(a1).toBe("a")
})
