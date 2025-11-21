import { lib } from "./lib";

it("should iterate `rules` first, then `oneOf`", () => {
	expect(lib).toEqual("abc02");
});
