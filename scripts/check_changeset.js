import "zx/globals";

import changesetsRead from "@changesets/read";

/**
 * @type {typeof changesetsRead}
 */
// @ts-expect-error the package seems not cooperate well with ESM
const readChangesets = changesetsRead.default;

await import("./meta/check_is_workspace_root.js");

async function checkVersion() {
	console.log("start checking version");
	const changesets = await readChangesets(process.cwd());
	console.log("validate version");

	/**
	 * @type {string[]}
	 */
	const errors = [];
	for (const changeset of changesets) {
		const { releases } = changeset;
		releases.forEach(release => {
			if (release.type === "major" || release.type === "minor") {
				errors.push(
					`- Packages "${release.name}" are not allowed to bump "${release.type}" version.`
				);
			}
		});
	}
	if (errors.length) {
		const messages = [
			'Rspack is currently using "0.x" as the version number, so major or minor version upgrades are not allowed yet.',
			"",
			...errors,
			"",
			'Please open your changeset file and modify the version type to "patch".'
		];
		console.log(messages.join("\n"));
		process.exit(1);
	} else {
		console.log("Check changeset version succeed.");
	}
}

async function checkBump() {
	try {
		const result = await $`pnpm changeset version`;
		if (result.exitCode !== 0) {
			console.error("Check changeset bump failed", result.stderr.toString());
			process.exit(1);
		}
		console.log("Check changeset bump succeed");
	} catch (err) {
		console.error("Check changeset bump failed");
		process.exit(1);
	}
}

await checkVersion();
// checkBump sometimes is hang forever so we take it down before it is fixed
//await checkBump(); 
