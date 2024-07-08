import { FooBar } from "./src/no-default";

it("no-default", () => {
	expect(FooBar).toBe("FooBar");
});
