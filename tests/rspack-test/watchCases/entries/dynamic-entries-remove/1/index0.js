import {value} from './shared'
it("should have correct entrypoints", function() {
  expect(Object.keys(__STATS__.entrypoints)).toEqual(["bundle0"]);

  const index0 = __STATS__.modules.find(m => m.name === "./index0.js");
  expect(index0.built).toBe(true);
  expect(index0.reasons.length).toBe(1);
  expect(index0.reasons[0].type).toBe("entry");

  const shared = __STATS__.modules.find(m => m.name === "./shared.js");
	expect(value).toBe(42)
	expect(shared.built).toBe(false);
  expect(shared.reasons.length).toBe(2);
})
