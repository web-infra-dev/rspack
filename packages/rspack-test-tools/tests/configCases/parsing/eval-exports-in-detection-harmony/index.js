import fs from "fs"; // import make this module a detection harmony

it("should compile", async () => {
	const a = typeof exports === "object";
	const file = await fs.promises.readFile(__filename, 'utf-8');
	expect(file).not.toContain(["const", "a", "=", "true"].join(" "))
	expect(file).toContain(["const", "a", "=", "typeof", "exports", "===", "\"object\""].join(" "))
});
