import content from "./loader.mjs!!";

it("should compile", () => {
	expect(typeof content).toBe("string");
	expect(content.startsWith("webpack://")).toBe(true);
});
