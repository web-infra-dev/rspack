import { runNodeCase } from "@rspack/test-tools/helper/node-case";

export default {
	description: "keeps chunk files current across asset mutations and compilations",
	async run() {
		await runNodeCase(new URL("./case.mjs", import.meta.url));
	}
};
