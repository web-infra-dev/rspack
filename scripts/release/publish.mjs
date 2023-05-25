import { getLastVersion } from "./version.mjs";
import * as core from "@actions/core";
export async function publish_handler(mode, options) {
	const npmrcPath = `${process.env.HOME}/.npmrc`;
	const root = process.cwd();
	if (fs.existsSync(npmrcPath)) {
		console.info("Found existing .npmrc file");
	} else {
		console.info(`No .npmrc file found, creating one`);

		fs.writeFileSync(
			npmrcPath,
			`//registry.npmjs.org/:_authToken=${process.env.NPM_TOKEN}`
		);
	}
	await $`pnpm publish -r ${options.dryRun ? "--dry-run" : ""} --tag ${
		options.tag
	} --no-git-checks`;
	const version = await getLastVersion(root);
	/**
	 * @Todo test stable release later
	 */
	if (mode === "stable") {
		const branch = core.getInput("branch");
		if (!branch) {
			throw new Error("should have branch env set");
		}
		console.info("git commit all...");
		await $`git status`;
		await $`git add .`;
		await $`git commit -m publish ${version}`;
		await $`git status`;
		await $`git push origin HEAD:${branch} --follow-tags`;
	}
}
