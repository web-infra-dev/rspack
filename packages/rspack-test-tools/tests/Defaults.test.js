const path = require("path");
const fs = require("fs");
const caseDir = path.resolve(__dirname, "./defaultsCases");
const {
	DefaultsConfigTaskProcessor,
	isDirectory,
	isValidCaseDirectory,
	createDefaultsCase
} = require("../dist");

describe("snapshots", () => {
	DefaultsConfigTaskProcessor.addSnapshotSerializer();
	const baseConfig = DefaultsConfigTaskProcessor.getDefaultConfig(
		__dirname,
		{ mode: "none" }
	);

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchSnapshot();
	});

	const categories = fs
		.readdirSync(caseDir)
		.filter(isValidCaseDirectory)
		.filter(folder => isDirectory(path.join(caseDir, folder)))
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.join(caseDir, cat))
					.filter(i => path.extname(i) === ".js")
					.sort()
			};
		});

	for (let cat of categories) {
		describe(cat.name, () => {
			for (let name of cat.tests) {
				createDefaultsCase(path.join(caseDir, cat.name, name));
			}
		});
	}
});
