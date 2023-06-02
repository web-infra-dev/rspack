import { loadRspackConfig } from "../../../src/utils/loadConfig";
import { resolve } from "path";

describe("loadRspackConfig", () => {
	it("should throw an error when config file does not exist", async () => {
		await expect(
			loadRspackConfig({
				config: resolve(__dirname, ".", "./non-existent-config.js")
			})
		).rejects.toThrow("config file");
	});

	it("should load test config file", async () => {
		const config = await loadRspackConfig({
			config: resolve(__dirname, ".", "test.rspack.config.cjs")
		});
		expect(config).toBeDefined();
	});
});
