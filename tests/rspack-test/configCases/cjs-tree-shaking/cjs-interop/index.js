import { colorToken } from "./a.js";

it("should correct resolve interop issue", () => {
	expect(colorToken.aaa).toBe("aaa");
});
