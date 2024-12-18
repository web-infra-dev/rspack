import { rspack } from "@rspack/core";
import { bench, describe } from "vitest";
import rspackConfig from "./fixtures/ts-react/rspack.config";

describe("TypeScript React project", () => {
	bench(
		"build single module project in production mode",
		() =>
			new Promise((resolve, reject) => {
				rspack(rspackConfig, (err, stats) => {
					if (err) {
						reject(err);
					}
					if (stats?.hasErrors()) {
						reject(new Error(stats.toString({})));
					}
					resolve();
				});
			})
	);
});
