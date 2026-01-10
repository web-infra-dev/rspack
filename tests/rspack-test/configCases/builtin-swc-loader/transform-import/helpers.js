import { FooBar } from "./src/helpers";

it("transformImport with various helpers (camelCase, snakeCase, upperCase, lowerCase)", () => {
	expect(FooBar).toBe("FooBar");
});
