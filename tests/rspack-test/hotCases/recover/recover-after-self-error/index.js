import getValue, { getError, id } from "./a";

const moduleId = id;

it("should abort when module is not accepted", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(getValue()).toBe(1);
	expect(getError()).toBe(false);
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(getValue()).toBe(2);
			expect(getError()).toBe(true);
			NEXT(
				require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
					expect(getValue()).toBe(2);
					expect(getError()).toBe(true);
					NEXT(
						require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
							expect(getValue()).toBe(4);
							expect(getError()).toBe(false);
							done();
						})
					);
				})
			);
		})
	);
}));

if (module.hot) {
	module.hot.accept("./a");
}
