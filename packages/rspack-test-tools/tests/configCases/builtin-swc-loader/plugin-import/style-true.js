import { FooBar, WithNumber3d } from "./src/style-true";

it("style-true", () => {
	expect(FooBar).toBe("FooBar");
	expect(WithNumber3d).toBe("WithNumber3d")
});
