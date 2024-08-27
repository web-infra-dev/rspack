describe("Validation", () => {
	const createTestCase = (name, config, fn, strategy, fn2) => {
		it(`should fail validation for ${name}`, () => {
			let prevStrategy = process.env.RSPACK_CONFIG_VALIDATE;
			process.env.RSPACK_CONFIG_VALIDATE = strategy;
			let errors = [];
			console.error = (...args) => {
				errors.push(...args);
			};
			try {
				const rspack = require("@rspack/core");
				rspack(config);
			} catch (err) {
				if (err.name !== "ValidationError") throw err;

				if (strategy === "loose") {
					throw new Error("Validation should not be failed in loose mode");
				}

				expect(err.message).toMatch(/^Configuration error:/);
				fn(err.message);

				return;
			} finally {
				if (strategy === "loose" || strategy === "loose-unrecognized-keys") {
					if (typeof fn2 !== "function") {
						throw new Error("Should provide a function for error testing");
					}
					fn2(errors);
				}
				process.env.RSPACK_CONFIG_VALIDATE = prevStrategy;
			}

			// Only in strict mode(default mode), we expect the validation always to fail
			// loose-unrecognized-keys and loose mode will ignore the additional properties and log a warning
			// loose-unrecognized-keys will fail the validation if the errors returned are not only unrecognized keys
			if (strategy === "strict" || !strategy) {
				throw new Error("Validation didn't fail");
			}
		});
	};

	describe("loose-unrecognized-keys", () => {
		createTestCase(
			"additional properties in loose-unrecognized-keys should be ignored",
			{
				context: "./",
				_additionalProperty: "test"
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
				"Configuration error:
				- The provided value \\"./\\" must be an absolute path. at \\"context\\""
			`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  "Configuration error:
			- Unrecognized key(s) in object: '_additionalProperty'",
			]
		`);
			}
		);

		createTestCase(
			"additional properties recursive in loose-unrecognized-keys should be ignored",
			{
				context: "./",
				optimization: {
					_additionalProperty: "test"
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
				"Configuration error:
				- The provided value \\"./\\" must be an absolute path. at \\"context\\""
			`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  "Configuration error:
			- Unrecognized key(s) in object: '_additionalProperty' at \\"optimization\\"",
			]
		`);
			}
		);

		createTestCase(
			"loose-unrecognized-keys should fail if the errors returned are other issues than unrecognized keys",
			{
				context: "./"
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
				"Configuration error:
				- The provided value \\"./\\" must be an absolute path. at \\"context\\""
			`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`Array []`);
			}
		);
	});

	describe("loose", () => {
		createTestCase(
			"should not be failed for any errors",
			{
				context: "./",
				_additionalProperty: "test",
				optimization: {
					_additionalProperty: "test"
				}
			},
			_ => {},
			"loose",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  "Configuration error:
			- The provided value \\"./\\" must be an absolute path. at \\"context\\"
			- Unrecognized key(s) in object: '_additionalProperty' at \\"optimization\\"
			- Unrecognized key(s) in object: '_additionalProperty'",
			]
				`);
			}
		);
	});

	describe("strict", () => {
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
			},
			"strict"
		);

		createTestCase(
			"unrecognized keys",
			{
				context: "./",
				_additionalProperty: "test",
				optimization: {
					_additionalProperty: "test"
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			"Configuration error:
			- The provided value \\"./\\" must be an absolute path. at \\"context\\"
			- Unrecognized key(s) in object: '_additionalProperty' at \\"optimization\\"
			- Unrecognized key(s) in object: '_additionalProperty'"
		`);
			},
			"strict"
		);
	});

	describe("default (strict)", () => {
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

		createTestCase(
			"unrecognized keys",
			{
				context: "./",
				_additionalProperty: "test",
				optimization: {
					_additionalProperty: "test"
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			"Configuration error:
			- The provided value \\"./\\" must be an absolute path. at \\"context\\"
			- Unrecognized key(s) in object: '_additionalProperty' at \\"optimization\\"
			- Unrecognized key(s) in object: '_additionalProperty'"
		`);
			}
		);
	});
});
