import { getName } from "./consumer";

it("should await an async external through an import-then-export barrel", () => {
	expect(getName()).toBe("file.txt");
});
