it("should allow to dynamic import a css module", async () => {
	require("./style_module_css.bundle0.js");
	await import("./style.module.css").then(x => {
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

it("should allow to reexport a css module", async () => {
	require("./reexported_js.bundle0.js");
	await import("./reexported").then(x => {
			expect(x).toEqual(
				nsObj({
					a: "a",
					abc: "a b c",
					comments: "abc      def",
					whitespace: "abc\n\tdef"
				})
			);
	});
});

it("should allow to import a css module", async () => {
	require("./imported_js.bundle0.js");
	await import("./imported").then(({ default: x }) => {
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
