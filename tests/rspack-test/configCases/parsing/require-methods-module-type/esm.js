export {};

it("should not enable require.context in javascript/esm", function () {
	expect(() => {
		require.context("./dir", false, /\.js$/);
	}).toThrowError(/require\.context is not a function/);
});

it("should not enable require.ensure in javascript/esm", function () {
	expect(() => {
		require.ensure(["./ensure"], function () {});
	}).toThrowError(/require\.ensure is not a function/);
});
