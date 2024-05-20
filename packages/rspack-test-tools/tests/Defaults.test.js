const path = require("path");
const {
	DefaultsConfigTaskProcessor,
	createDefaultsCase,
	describeByWalk
} = require("..");

DefaultsConfigTaskProcessor.addSnapshotSerializer(expect);

describe("Base Defaults Snapshot", () => {
	const baseConfig = DefaultsConfigTaskProcessor.getDefaultConfig(
		path.resolve(__dirname, ".."),
		{ mode: "none" }
	);

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchSnapshot();
	});
});

describeByWalk(__filename, (name, src, dist) => {
	createDefaultsCase(name, src);
}, {
	type: "file",
});
