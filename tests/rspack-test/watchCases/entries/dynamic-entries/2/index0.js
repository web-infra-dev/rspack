it("should have correct entrypoints", function() {
  expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0", "bundle1", "bundle2"]);

  const index0 = __STATS__.modules.find(m => m.name === "./index0.js");
  expect(index0.built).toBe(true);
  expect(index0.reasons.length).toBe(1);
  expect(index0.reasons[0].type).toBe("entry");

  const index1 = __STATS__.modules.find(m => m.name === "./index1.js");
  expect(index1.built).toBe(false);
  expect(index1.reasons.length).toBe(1);
  expect(index1.reasons[0].type).toBe("entry");

  const index2 = __STATS__.modules.find(m => m.name === "./index2.js");
  expect(index2.built).toBe(true);
  expect(index2.reasons.length).toBe(1);
  expect(index2.reasons[0].type).toBe("entry");
})
