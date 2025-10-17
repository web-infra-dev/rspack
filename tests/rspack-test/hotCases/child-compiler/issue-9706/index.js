import value, { assets } from "./report-child-assets-loader!./file";

it("should not emit hot updates from child compilers", async () => {
	expect(value).toBe(1);
	expect(assets).toEqual(["test.js"]);
	await NEXT_HMR();
	expect(value).toBe(2);
	expect(assets).toEqual(["test.js"]);
});

module.hot.accept("./report-child-assets-loader!./file");