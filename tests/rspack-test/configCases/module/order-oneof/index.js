import { lib } from "./lib";

it("should run oneOf loaders first", () => {
	expect(lib).toEqual("abc10");
});
