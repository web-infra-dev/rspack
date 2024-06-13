const path = require("path");
const {
	DefaultsConfigProcessor,
	createDefaultsCase,
	describeByWalk
} = require("..");

function getWebpackDefaultConfig(cwd, config) {
	const { applyWebpackOptionsDefaults, getNormalizedWebpackOptions } =
		require("webpack").config;
	config = getNormalizedWebpackOptions(config);
	applyWebpackOptionsDefaults(config);
	process.chdir(cwd);
	return config;
}

function getObjectPaths(obj, parentPaths = []) {
	return Object.keys(obj).reduce((paths, key) => {
		const fullPath = [...parentPaths, key];
		if (Array.isArray(obj[key]) && obj[key].length) {
			return [...paths, fullPath, ...getObjectPaths(obj[key][0], fullPath)];
		} else if (typeof obj[key] === "object" && obj[key] !== null) {
			return [...paths, fullPath, ...getObjectPaths(obj[key], fullPath)];
		} else {
			return [...paths, fullPath];
		}
	}, []);
}

function filterObjectPaths(obj, paths, parentPaths = []) {
	for (const key of Object.keys(obj)) {
		const fullPath = [...parentPaths, key];
		if (!paths.some(p => p.length === fullPath.length && p.every((e, i) => e === fullPath[i]))) {
			delete obj[key];
			continue;
		}
		if (Array.isArray(obj[key])) {
			for (const item of obj[key]) {
				filterObjectPaths(item, paths, fullPath);
			}
		} else if (typeof obj[key] === "object" && obj[key] !== null) {
			filterObjectPaths(obj[key], paths, fullPath);
		}
	}
}

DefaultsConfigProcessor.addSnapshotSerializer(expect);

const cwd = path.resolve(__dirname, "..");

describe("Base Defaults Snapshot", () => {
	const baseConfig = DefaultsConfigProcessor.getDefaultConfig(cwd, { mode: "none" });

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchSnapshot();
	});

	it("should be align to webpack base config", () => {
		const webpackBaseConfig = getWebpackDefaultConfig(cwd, { mode: "none" });
		const rspackSupportedConfig = getObjectPaths(baseConfig);
		filterObjectPaths(webpackBaseConfig, rspackSupportedConfig);
		expect(baseConfig).toEqual(webpackBaseConfig);
	});
});

describeByWalk(__filename, (name, src, dist) => {
	createDefaultsCase(name, src);
}, {
	type: "file",
});
