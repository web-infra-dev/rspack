import { FooBar } from "./src/named-import";

it("transformImport with namedImport: true", () => {
	expect(FooBar).toBe("FooBar");
});
