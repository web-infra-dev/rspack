it("should fire multiple code load callbacks in the correct order", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	var calls = [];
	require.ensure([], function(require) {
		require("./duplicate");
		require("./duplicate2");
		calls.push(1);
	});
	require.ensure([], function(require) {
		require("./duplicate");
		require("./duplicate2");
		calls.push(2);
		expect(calls).toEqual([1,2]);
		done();
	});
}));
