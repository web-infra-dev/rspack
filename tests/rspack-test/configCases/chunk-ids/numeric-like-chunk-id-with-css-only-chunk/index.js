it("should load the JavaScript of a chunk whose id coerces to the id of a css-only chunk", async () => {
  const { value } = await import(/* webpackChunkName: "00" */ "./chunk");
  expect(value).toBe("chunk");
});
