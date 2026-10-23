import path from "node:path";

const root = path.resolve(import.meta.dirname, "../../../..");

/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
export default {
	description: "portable",
	options: () => ({ cache: { type: "persistent", portable: true } }),
	diff: e => e.toMatchObject({
		value: expect.stringContaining(`"root": ${JSON.stringify(root)}`)
	})
};
