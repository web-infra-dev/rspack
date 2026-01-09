import { FooBar } from "./src/default-import";

it("transformImport with namedImport: false (default import)", () => {
	expect(FooBar).toBe("FooBar");
});
