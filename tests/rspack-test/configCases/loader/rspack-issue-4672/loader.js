const assert = require("assert");

module.exports = function () {
	const callback = this.async();

	this.resolve(__dirname, "./not-exist", (e1, r1) => {
		// Note that e1 is not instanceof Error
		assert(e1.code === "GenericFailure");
		assert(typeof e1.message === 'string')
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
