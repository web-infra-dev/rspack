describe("Validation", () => {
	const createTestCase = (name, config, fn) => {
		it(`should fail validation for ${name}`, () => {
			try {
				const rspack = require("@rspack/core");
				rspack(config);
			} catch (err) {
				if (err.name !== "ValidationError") throw err;

				expect(err.message).toMatch(/^Configuration error:/);
				fn(err.message);

				return;
			}

			throw new Error("Validation didn't fail");
		});
	};

	createTestCase(
		"not absolute context",
		{
			context: "./"
		},
		message => {
			expect(message).toMatchInlineSnapshot(`
			"Configuration error:
			- The provided value \\"./\\" must be an absolute path. at \\"context\\""
		`);
		}
	);
});
