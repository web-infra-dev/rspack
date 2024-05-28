const path = require("path");
const {
	DefaultsConfigProcessor,
	createDefaultsCase,
	describeByWalk
} = require("..");

DefaultsConfigProcessor.addSnapshotSerializer(expect);

describe("Base Defaults Snapshot", () => {
	const baseConfig = DefaultsConfigProcessor.getDefaultConfig(
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
