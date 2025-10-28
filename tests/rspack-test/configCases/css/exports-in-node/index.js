import * as style from "../pseudo-export/style.module.css?ns";
import { a, abc } from "../pseudo-export/style.module.css?picked";
import def from "../pseudo-export/style.module.css?default";

it("should allow to import a css module", () => {
	expect(style).toEqual(
		nsObj({
			a: "a",
			abc: "a b c",
			comments: "abc      def",
			whitespace: "abc\n\tdef",
			default: "default"
		})
	);
	expect(a).toBe("a");
	expect(abc).toBe("a b c");
	expect(def).toBe("default");
});

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

it("should allow to reexport a css module", async () => {
	await import("../pseudo-export/reexported").then(x => {
			expect(x).toEqual(
				nsObj({
					a: "a",
					abc: "a b c",
					comments: "abc      def",
					whitespace: "abc\n\tdef",
				})
			);
	});
});

it("should allow to import a css module", async () => {
	await import("../pseudo-export/imported").then(({ default: x }) => {
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
