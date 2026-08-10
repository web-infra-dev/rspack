it("should expose a callable require.ensure callback without a dispatcher", () =>
	new Promise((resolve, reject) => {
		require.ensure(
			["./value"],
			(require) => {
				expect(typeof require).toBe("function");
				expect(require("./value")).toBe(44);
				resolve();
			},
			(error) => reject(error),
		);
	}));
