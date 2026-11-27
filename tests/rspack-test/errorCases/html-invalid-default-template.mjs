import path from "node:path";
import { HtmlRspackPlugin } from "@rspack/core";

const context = path.resolve(import.meta.dirname, "../fixtures/errors/html-invalid-template");

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description: "should report invalid UTF-8 in the default HTML template",
	options() {
		return {
			context,
			entry: {},
			plugins: [new HtmlRspackPlugin()]
		};
	},
	check({ errors }) {
		expect(errors).toHaveLength(1);
		expect(errors[0].message).toContain(
			`HtmlRspackPlugin: Failed to read template '${path.join(context, "src/index.ejs")}': stream did not contain valid UTF-8`
		);
	}
};
