import { lib } from "./lib";

it("should run nested rules loader first, then oneOf loader, then parent rule", () => {
	expect(lib).toEqual("abc210");
});
