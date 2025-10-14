it("should compile to lazy imported module", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	let resolved;
	const promise = import("./module").then(r => (resolved = r));
	let generation = 0;
	module.hot.accept("./module", () => {
		generation++;
	});
	expect(resolved).toBe(undefined);
	setTimeout(() => {
		expect(resolved).toBe(undefined);
		expect(generation).toBe(0);
		NEXT(
			require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
				promise.then(result => {
					expect(result).toHaveProperty("default", 42);
					expect(generation).toBe(0);
					NEXT(
						require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
							expect(result).toHaveProperty("default", 42);
							expect(generation).toBe(1);
							import("./module").then(result => {
								expect(result).toHaveProperty("default", 43);
								expect(generation).toBe(1);
								module.hot.accept("./module", () => {
									generation += 10;
								});
								NEXT(
									require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
										import("./module").then(result => {
											expect(result).toHaveProperty("default", 44);
											expect(generation).toBe(11);
											setTimeout(() => {
												done();
											}, 1000);
										}, done);
									})
								);
							}, done);
						})
					);
				}, done);
			})
		);
	}, 1000);
}));
