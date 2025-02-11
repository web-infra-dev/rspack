it("should have correct entrypoints", function() {
  expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0", "bundle1"]);
})
