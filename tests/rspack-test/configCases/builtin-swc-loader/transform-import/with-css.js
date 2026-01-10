import { FooBar } from "./src/with-css";

it("transformImport with CSS side-effect", () => {
	expect(FooBar).toBe("FooBar");
});
