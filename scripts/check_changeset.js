const { spawnSync } = require("child_process");
const path = require("path");
const util = require("util");
const readChangesets = require("@changesets/read").default;

async function checkVersion() {
	const changesets = await readChangesets(path.join(__dirname, "../"));

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
	const result = spawnSync("pnpm", ["changeset", "version"], {
		stdio: "pipe"
	});
	if (result.status !== 0) {
		console.error("Check changeset bump failed", result.stderr.toString());
		process.exit(1);
	} else {
		console.log("Check changeset bump succeed");
	}
}
async function main() {
	await checkVersion();
	await checkBump();
}

main().catch(err => {
	console.error("check changeset failed", err);
	process.exit(1);
});
