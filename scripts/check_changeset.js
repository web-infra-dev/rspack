const path = require("path");
const readChangesets = require("@changesets/read").default;

async function run() {
	const cwd = process.cwd();
	const changesets = await readChangesets(
		path.join(cwd, "../"),
		process.env.BASE_BRANCH
	);

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

	return errors;
}

run().then(errors => {
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
	}
});
