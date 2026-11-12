const modules = import.meta.glob("./SRC/COMPONENTS/*.JS", {
	caseSensitive: false,
	eager: true
});

it("should only visit directories related to the literal glob prefix", () => {
	expect(Object.keys(modules)).toEqual(["./src/components/alpha.js"]);
	expect(modules["./src/components/alpha.js"].default).toBe("alpha");
});
