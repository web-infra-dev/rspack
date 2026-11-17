import path from "node:path";
import { config } from "@rspack/core";

const root = path.resolve(import.meta.dirname, "../../../..");

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "portable-experiment",
	options: () => ({
		cache: { type: "persistent" },
		experiments: { portableCache: true }
	}),
	diff: (e) => {
		e.toMatchObject({
			value: expect.stringContaining(`"root": ${JSON.stringify(root)}`)
		});
		e.toMatchObject({ value: expect.stringContaining('"seed": true') });
		expect(() => {
			const options = config.getNormalizedRspackOptions({
				cache: { type: "persistent", portable: false },
				experiments: { portableCache: true }
			});
			config.applyRspackOptionsDefaults(options);
		}).toThrow("experiments.portableCache requires cache.portable");
	}
};
