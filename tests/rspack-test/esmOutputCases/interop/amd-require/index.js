it("should initialize AMD dependencies without a module dispatcher", () =>
	new Promise((resolve, reject) => {
		require(
			["./value"],
			(value) => {
				expect(value).toBe(42);
				resolve();
			},
			reject,
		);
	}));

it("should expose a callable AMD require without a module dispatcher", () =>
	new Promise((resolve) => {
		define(["require"], (localRequire) => {
			expect(typeof localRequire).toBe("function");
			expect(localRequire("./sync-value")).toBe(43);
			resolve();
		});
	}));
