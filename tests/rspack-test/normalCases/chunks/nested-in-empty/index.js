it("should include a chunk nested in an empty chunk", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(["./a"], () => {
		require.ensure([], () => {
			require.ensure(["./a"], () => {
				require.ensure([], () => {
					const b = require("./b");
					expect(b).toBe(42);
					done();
				});
			});
		});
	});
}));
