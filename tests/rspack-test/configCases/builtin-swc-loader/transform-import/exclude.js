import { Included, Excluded } from "./src/exclude";

it("transformImport with exclude", () => {
	expect(Included).toBe("Included");
	// Excluded should still work since it's not transformed but stays as named export from source
	expect(Excluded).toBe("Excluded");
});
