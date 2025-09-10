it("should have correct entrypoints", function() {
  expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0"]);

  const index0 = __STATS__.modules.find(m => m.name === "./index0.js");
  expect(index0.built).toBe(true);
  expect(index0.reasons.length).toBe(1);
  expect(index0.reasons[0].type).toBe("entry");
})
