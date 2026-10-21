/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
	name: "no-resend-on-dispose",
	description: "disposing a lazy compilation proxy must not repost the remaining active ids",
	async run() {
		const { run } = await import("./client.mjs");
		await run();
	}
};
