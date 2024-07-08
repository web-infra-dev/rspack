import * as path from "path";
import { fileURLToPath } from "url";
import * as core from "@actions/core";

import { getLastVersion } from "./version.mjs";

const __filename = path.resolve(fileURLToPath(import.meta.url));
const __dirname = path.dirname(__filename);

export async function publish_handler(mode, options) {
	console.log("options:", options);
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
	} --no-git-checks --provenance`;
	const version = await getLastVersion(root);
	core.setOutput("version", version);
	core.notice(`Version: ${version}`);
	// write version to workspace directory
	fs.writeFileSync(path.resolve(__dirname, "../..", "version_output"), version);
	/**
	 * @Todo test stable release later
	 */
	if (options.pushTags) {
		console.info("git config user");
		await $`git config --global --add safe.directory /github/workspace`;
		await $`git config --global user.name "github-actions[bot]"`;
		await $`git config --global user.email "github-actions[bot]@users.noreply.github.com"`;
		console.info("git commit all...");
		await $`git status`;
		await $`git tag v${version} -m v${version} `;
		await $`git push origin --follow-tags`;
	}
}
