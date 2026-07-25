const { take: takeA } = require("./module-a");
const { take: takeB } = require("./module-b");

it("should share TypeScript iterable read fallbacks", () => {
	expect(takeA(new Set(["a", "b", "c"]), 2)).toEqual(["a", "b"]);
	expect(takeB(["d", "e", "f"], 1)).toEqual(["d"]);
});
