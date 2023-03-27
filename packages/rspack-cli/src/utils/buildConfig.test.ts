import { RspackOptions } from "@rspack/core";
import { RspackCLIOptions } from "../types";
import { buildConfigWithOptions } from "./buildConfig";

describe("buildConfigWithOptions", () => {
	it("should set mode by NODE_ENV", () => {
		process.env.NODE_ENV = "production";
		const options: RspackCLIOptions = {};
		const item: RspackOptions = { mode: undefined };
		buildConfigWithOptions(item, options, true);
		expect(item.mode).toBe("production");
	});

	it("should set mode by cli options", () => {
		// --mode option on the CLI takes precedence over mode in rspack.config.js
		const options: RspackCLIOptions = { mode: "production" };
		const item: RspackOptions = { mode: "none" };
		buildConfigWithOptions(item, options, true);
		expect(item.mode).toBe("production");
	});

	it("should set watch by cli options", () => {
		const options: RspackCLIOptions = { watch: true };
		const item: RspackOptions = {};
		buildConfigWithOptions(item, options, true);
		expect(item.watch).toBe(true);
	});
});
