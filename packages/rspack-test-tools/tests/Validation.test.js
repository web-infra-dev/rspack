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
				const { rspack } = require("@rspack/core");
				rspack(config);
			} catch (err) {
				if (err.name !== "ValidationError") throw err;

				if (strategy === "loose") {
					throw new Error("Validation should not be failed in loose mode");
				}

				expect(err.message).toMatch(/^Invalid configuration object/);
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
			"should not be failed for builtin:swc-loader errors",
			{
				module: {
					rules: [
						{
							test: /\.js$/,
							use: {
								loader: "builtin:swc-loader",
								options: {
									_additionalProperty: "foo",
									rspackExperiments: {
										_additionalProperty: "bar"
									}
								}
							}
						},
						{
							test: /\.js$/,
							loader: "builtin:swc-loader",
							options: {
								_additionalProperty: "foo",
								rspackExperiments: {
									_additionalProperty: "bar"
								}
							}
						}
					]
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use"
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1]"
		`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options",
			]
		`);
			}
		);

		createTestCase(
			"additional properties in loose-unrecognized-keys should be ignored",
			{
				_additionalProperty: "test"
			},
			message => {
				throw new Error("should not have error");
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty',
			]
		`);
			}
		);

		createTestCase(
			"additional properties recursive in loose-unrecognized-keys should be ignored",
			{
				optimization: {
					_additionalProperty: "test"
				}
			},
			message => {
				throw new Error("should not have error");
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "optimization",
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
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
		`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`Array []`);
			}
		);

		createTestCase(
			"loose-unrecognized-keys should print warning and error at the same time if both kinds of errors are returned",
			{
				context: "./",
				_additionalProperty: "test"
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
		`);
			},
			"loose-unrecognized-keys",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty',
			]
		`);
			}
		);
	});

	describe("loose", () => {
		createTestCase(
			"should not be failed for builtin:swc-loader errors",
			{
				module: {
					rules: [
						{
							test: /\.js$/,
							use: {
								loader: "builtin:swc-loader",
								options: {
									_additionalProperty: "foo",
									rspackExperiments: {
										_additionalProperty: "bar"
									}
								}
							}
						},
						{
							test: /\.js$/,
							loader: "builtin:swc-loader",
							options: {
								_additionalProperty: "foo",
								rspackExperiments: {
									_additionalProperty: "bar"
								}
							}
						}
					]
				}
			},
			message => {
				throw new Error("should not have error");
			},
			"loose",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options"
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options"
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1]",
			]
		`);
			}
		);

		createTestCase(
			"should not be failed for wrong externals with output.libraryTarget: umd",
			{
				externals: [
					{
						foo: {
							commonjs2: "foo"
						}
					}
				],
				output: {
					library: {
						type: "umd"
					}
				}
			},
			message => {
				throw new Error("should not have error");
			},
			"loose",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Expected string, received object at "externals[0]", or Input not instance of RegExp at "externals[0]", or Expected string, received object at "externals[0].foo", or Expected boolean, received object at "externals[0].foo", or Expected array, received object at "externals[0].foo", or Required at "externals[0].foo.root"
			- Required at "externals[0].foo.commonjs"
			- Required at "externals[0].foo.amd"
			- External object must have "root", "commonjs", "commonjs2", "amd" properties when "libraryType" or "externalsType" is "umd" at "externals[0].foo", or Expected function, received object at "externals[0]", or Expected function, received object at "externals[0]", or Expected function, received object at "externals[0]", or Expected string, received array at "externals", or Input not instance of RegExp at "externals", or Expected object, received array at "externals", or Expected function, received array at "externals", or Expected function, received array at "externals", or Expected function, received array at "externals",
			]
		`);
			}
		);

		createTestCase(
			"should not be failed for any errors",
			{
				context: "./",
				_additionalProperty: "test",
				optimization: {
					_additionalProperty: "test"
				}
			},
			message => {
				throw new Error("should not have error");
			},
			"loose",
			log => {
				expect(log).toMatchInlineSnapshot(`
			Array [
			  Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
			- Unrecognized key(s) in object: '_additionalProperty' at "optimization"
			- Unrecognized key(s) in object: '_additionalProperty',
			]
		`);
			}
		);
	});

	describe("strict", () => {
		createTestCase(
			"should failed for builtin:swc-loader errors",
			{
				module: {
					rules: [
						{
							test: /\.js$/,
							use: {
								loader: "builtin:swc-loader",
								options: {
									_additionalProperty: "foo",
									rspackExperiments: {
										_additionalProperty: "bar"
									}
								}
							}
						},
						{
							test: /\.js$/,
							loader: "builtin:swc-loader",
							options: {
								_additionalProperty: "foo",
								rspackExperiments: {
									_additionalProperty: "bar"
								}
							}
						}
					]
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use.options"
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[0].use"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options.rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1].options"
			- Invalid options of 'builtin:swc-loader': Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Unrecognized key(s) in object: '_additionalProperty' at "rspackExperiments"
			- Unrecognized key(s) in object: '_additionalProperty' at "module.rules[1]"
		`);
			},
			"strict",
			log => {
				throw new Error("should not have log");
			}
		);

		createTestCase(
			"should not be failed for wrong externals with output.libraryTarget: umd",
			{
				externals: [
					{
						foo: {
							commonjs2: "foo"
						}
					}
				],
				output: {
					library: {
						type: "umd"
					}
				}
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- Expected string, received object at "externals[0]", or Input not instance of RegExp at "externals[0]", or Expected string, received object at "externals[0].foo", or Expected boolean, received object at "externals[0].foo", or Expected array, received object at "externals[0].foo", or Required at "externals[0].foo.root"
			- Required at "externals[0].foo.commonjs"
			- Required at "externals[0].foo.amd"
			- External object must have "root", "commonjs", "commonjs2", "amd" properties when "libraryType" or "externalsType" is "umd" at "externals[0].foo", or Expected function, received object at "externals[0]", or Expected function, received object at "externals[0]", or Expected function, received object at "externals[0]", or Expected string, received array at "externals", or Input not instance of RegExp at "externals", or Expected object, received array at "externals", or Expected function, received array at "externals", or Expected function, received array at "externals", or Expected function, received array at "externals"
		`);
			},
			"strict",
			log => {
				throw new Error("should not have log");
			}
		);

		createTestCase(
			"not absolute context",
			{
				context: "./"
			},
			message => {
				expect(message).toMatchInlineSnapshot(`
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
		`);
			},
			"strict",
			log => {
				throw new Error("should not have log");
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
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
			- Unrecognized key(s) in object: '_additionalProperty' at "optimization"
			- Unrecognized key(s) in object: '_additionalProperty'
		`);
			},
			"strict",
			log => {
				throw new Error("should not have log");
			}
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
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
		`);
			},
			log => {
				throw new Error("should not have log");
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
			Invalid configuration object. Rspack has been initialized using a configuration object that does not match the API schema.
			- The provided value "./" must be an absolute path. at "context"
			- Unrecognized key(s) in object: '_additionalProperty' at "optimization"
			- Unrecognized key(s) in object: '_additionalProperty'
		`);
			},
			log => {
				throw new Error("should not have log");
			}
		);
	});
});
