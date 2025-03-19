const BOOTSTRAP_SPLIT_LINE =
	"/************************************************************************/";
const MODULE_START_FLAG = "/* start::";
const MODULE_END_FLAG = "/* end::";
const MODULE_FLAG_END = " */";

function getStringBetween(
	raw: string,
	position: number,
	start: string,
	end: string
) {
	const startFlagIndex = raw.indexOf(start, position);
	if (startFlagIndex === -1) {
		return {
			result: null,
			remain: position
		};
	}
	const endFlagIndex = raw.indexOf(end, startFlagIndex + start.length);
	if (endFlagIndex === -1) {
		return {
			result: null,
			remain: position
		};
	}
	return {
		result: raw.slice(startFlagIndex + start.length, endFlagIndex),
		remain: endFlagIndex + end.length
	};
}

function isValidModule(name: string) {
	if (name.startsWith("data:")) {
		return false;
	}
	if (name.startsWith("https:")) {
		return false;
	}
	return true;
}

export function parseModules(
	content: string,
	options: {
		bootstrap?: boolean;
		renameModule?: (name: string) => string;
	} = {}
) {
	const modules: Record<string, string> = {};
	const runtimeModules: Record<string, string> = {};

	let currentPosition = 0;

	if (options.bootstrap) {
		// parse bootstrap code
		const bootstrap = getStringBetween(
			content,
			0,
			BOOTSTRAP_SPLIT_LINE,
			BOOTSTRAP_SPLIT_LINE
		);
		if (bootstrap.result) {
			runtimeModules["webpack/runtime/bootstrap"] = bootstrap.result;
		}
	}
	// parse module & runtime module code
	let moduleName = getStringBetween(
		content,
		currentPosition,
		MODULE_START_FLAG,
		MODULE_FLAG_END
	).result;
	while (moduleName) {
		const moduleContent = getStringBetween(
			content,
			currentPosition,
			`${MODULE_START_FLAG}${moduleName}${MODULE_FLAG_END}`,
			`${MODULE_END_FLAG}${moduleName}${MODULE_FLAG_END}`
		);
		if (!moduleContent.result) {
			throw new Error(`Module code parsed error: ${moduleName}`);
		}
		const renamedModuleName =
			typeof options.renameModule === "function"
				? options.renameModule(moduleName)
				: moduleName;
		if (moduleName.startsWith("webpack/runtime")) {
			runtimeModules[renamedModuleName] = moduleContent.result;
		} else {
			if (isValidModule(moduleName)) {
				modules[renamedModuleName] = moduleContent.result;
			}
		}
		currentPosition = moduleContent.remain;
		moduleName = getStringBetween(
			content,
			currentPosition,
			MODULE_START_FLAG,
			MODULE_FLAG_END
		).result;
	}
	return {
		modules,
		runtimeModules
	};
}
