import { lib } from "./lib";

it("should run nested rules loader first", () => {
	expect(lib).toEqual("abc012");
});
