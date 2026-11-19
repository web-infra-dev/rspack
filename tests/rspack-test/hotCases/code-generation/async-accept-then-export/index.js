import value from "./module";

it("should preserve the namespace of an async dependency exporting then", async () => {
	expect(value).toBe(1);
	let accepted;
	import.meta.webpackHot.accept("./module", () => { accepted = value; });
	await NEXT_HMR();
	expect(accepted).toBe(2);
	expect(value).toBe(2);
});
