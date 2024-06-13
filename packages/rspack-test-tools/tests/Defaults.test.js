const path = require("path");
const fs = require("fs");
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

function trimObjectPaths(obj, paths, parentPaths = []) {
	for (const key of Object.keys(obj)) {
		const fullPath = [...parentPaths, key];
		if (paths.some(p => p.length === fullPath.length && p.every((e, i) => e === fullPath[i]))) {
			delete obj[key];
			continue;
		}
		if (Array.isArray(obj[key])) {
			for (const item of obj[key]) {
				trimObjectPaths(item, paths, fullPath);
			}
		} else if (typeof obj[key] === "object" && obj[key] !== null) {
			trimObjectPaths(obj[key], paths, fullPath);
		}
	}
}

DefaultsConfigProcessor.addSnapshotSerializer(expect);

const cwd = path.resolve(__dirname, "..");

function assertWebpackConfig(config) {
	const rspackBaseConfig = DefaultsConfigProcessor.getDefaultConfig(cwd, config);
	const webpackBaseConfig = getWebpackDefaultConfig(cwd, config);
	const rspackSupportedConfig = getObjectPaths(rspackBaseConfig);
	const defaultsPath = path.resolve(__dirname, "../../rspack/src/config/defaults.ts");
	const defaultsContent = fs.readFileSync(defaultsPath, "utf-8");
	const regex = /\/\/\sIGNORE\((.+?)\):\s/g;
	const ignoredPaths = [];
	let matches;
	while (matches = regex.exec(defaultsContent)) {
		ignoredPaths.push(matches[1].split('.'));
	}
	trimObjectPaths(rspackBaseConfig, ignoredPaths);
	trimObjectPaths(webpackBaseConfig, ignoredPaths);
	filterObjectPaths(webpackBaseConfig, rspackSupportedConfig);
	expect(rspackBaseConfig).toEqual(webpackBaseConfig);
}

describe("Base Defaults Snapshot", () => {
	const baseConfig = DefaultsConfigProcessor.getDefaultConfig(cwd, { mode: "none" });

	it("should have the correct base config", () => {
		expect(baseConfig).toMatchSnapshot();
	});

	it("should be align to webpack base config for mode: none", () => {
		assertWebpackConfig({ mode: "none" });
	});

	it("should be align to webpack base config for mode: development", () => {
		assertWebpackConfig({ mode: "development" });
	});

	it("should be align to webpack base config for mode: production", () => {
		assertWebpackConfig({ mode: "production" });
	});

	it("should be align to webpack base config for experiments.futureDefaults: true", () => {
		assertWebpackConfig({
			mode: "production", experiments: {
				futureDefaults: true
			}
		});
	});
});

describeByWalk(__filename, (name, src, dist) => {
	createDefaultsCase(name, src);
}, {
	type: "file",
});
