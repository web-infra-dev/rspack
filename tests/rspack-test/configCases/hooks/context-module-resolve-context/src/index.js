it("should preserve the resolve context through context module factory hooks", () => {
  const name = "value";
  expect(require(`./local/${name}`).default).toBe(42);
});
