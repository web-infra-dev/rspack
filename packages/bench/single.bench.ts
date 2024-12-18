import { join } from "node:path";
import { bench, describe } from "vitest";
import { rspack } from "@rspack/core";

describe("single module project", () => {
	bench(
		"build in production mode",
		() =>
			new Promise((resolve, reject) => {
				rspack(
					{
						context: join(__dirname, "fixtures/single"),
						entry: "index.js",
						mode: "production"
					},
					(err, stats) => {
						if (err) {
							reject(err);
						}
						if (stats?.hasErrors()) {
							reject(new Error(stats.toString({})));
						}
						resolve();
					}
				);
			})
	);
});
