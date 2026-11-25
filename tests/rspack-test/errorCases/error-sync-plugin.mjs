import assert from "node:assert/strict";
import throwErrorPlugin from "../fixtures/errors/throw-error-plugin.mjs";

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description: "should thrown sync error from plugin",
	options() {
		return {
			entry: "./no-errors-deprecate",
			plugins: [throwErrorPlugin]
		};
	},
	async check({ errors, warnings }) {
		assert.equal(errors.length, 1);
		assert.deepEqual(warnings, []);

		const [error] = errors;
		const pluginLocation = /[/\\]fixtures[/\\]errors[/\\]throw-error-plugin\.mjs:\d+:\d+/;
		assert.equal(error.code, "GenericFailure");
		assert.match(error.message, /^\s*× Error: foo\r?\n/);
		assert.match(error.message, pluginLocation);
		assert.match(error.stack, /^Error: foo\r?\n/);
		assert.match(error.stack, pluginLocation);
	}
};
