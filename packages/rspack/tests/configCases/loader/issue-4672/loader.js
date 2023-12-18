const assert = require("assert");

module.exports = function () {
	const callback = this.async();

	this.resolve(__dirname, "./not-exist", (e1, r1) => {
		assert(e1 instanceof Error);
		assert(typeof r1 === "undefined");

		this.resolve(__dirname, "@/src/index", (e2, r2) => {
			assert(e2 === null);
			assert(r2.endsWith("index.js"));

			const resolve = this.getResolve({
				extensions: [".ts"]
			});
			resolve(__dirname, "./index", (e3, r3) => {
				assert(e3 === null);
				assert(r3.endsWith("index.ts"));

				callback(
					null,
					`
it("should use native resolver", () => {
  expect(1 + 1).toBe(2)
})`
				);
			});
		});
	});
};
