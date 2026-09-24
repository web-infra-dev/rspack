/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	name: "disposal",
	description:
		"disposing a lazy compilation proxy must not resend the remaining active module ids",
	async run() {
		const { run } = await import("./client.mjs");
		await run();
	}
};
