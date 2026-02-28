it("should allow to dynamic import a css module", async () => {
	await import("../pseudo-export/style.module.css").then(x => {
		expect(x).toEqual(
			nsObj({
				a: "a",
				abc: "a b c",
				comments: "abc      def",
				whitespace: "abc\n\tdef",
				default: "default"
			})
		);
	});
});

it("should allow to dynamic import a pure css", async () => {
	await import("./style.css").then(x => {
		expect(Object.keys(x).length).toBe(0)
	});
});
