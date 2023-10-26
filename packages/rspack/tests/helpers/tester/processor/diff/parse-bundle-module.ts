const BOOTSTRAP_SPLIT_LINE =
	"/************************************************************************/";
const MODULE_START_FLAG = "/* start::";
const MODULE_END_FLAG = "/* end::";
const MODULE_FLAG_END = " */";

function getStringBetween(raw: string, start: string, end: string) {
	const startFlagIndex = raw.indexOf(start);
	if (startFlagIndex === -1) {
		return {
			result: null,
			remain: raw
		};
	}
	const endFlagIndex = raw.slice(startFlagIndex + start.length).indexOf(end);
	if (endFlagIndex === -1) {
		return {
			result: null,
			remain: raw
		};
	}
	return {
		result: raw.slice(
			startFlagIndex + start.length,
			startFlagIndex + start.length + endFlagIndex
		),
		remain:
			raw.slice(0, startFlagIndex) +
			raw.slice(startFlagIndex + start.length + endFlagIndex + end.length)
	};
}

export function parseBundleModules(content: string) {
	const modules: Map<string, string> = new Map();
	const runtimeModules: Map<string, string> = new Map();

	// parse bootstrap code
	const bootstrap = getStringBetween(
		content,
		BOOTSTRAP_SPLIT_LINE,
		BOOTSTRAP_SPLIT_LINE
	);
	if (bootstrap.result) {
		runtimeModules.set("webpack/bootstrap", bootstrap.result);
		content = bootstrap.remain;
	}
	// parse module & runtime module code
	let moduleName = getStringBetween(
		content,
		MODULE_START_FLAG,
		MODULE_FLAG_END
	);
	while (moduleName.result) {
		const moduleContent = getStringBetween(
			content,
			`${MODULE_START_FLAG}${moduleName.result}${MODULE_FLAG_END}`,
			`${MODULE_END_FLAG}${moduleName.result}${MODULE_FLAG_END}`
		);
		if (!moduleContent.result) {
			throw new Error(`Module code parsed error: ${moduleName.result}`);
		}
		if (moduleName.result.startsWith("webpack/runtime")) {
			runtimeModules.set(moduleName.result, moduleContent.result);
		} else {
			modules.set(moduleName.result, moduleContent.result);
		}
		content = moduleContent.remain;
		moduleName = getStringBetween(content, MODULE_START_FLAG, MODULE_FLAG_END);
	}
	return {
		modules,
		runtimeModules
	};
}
