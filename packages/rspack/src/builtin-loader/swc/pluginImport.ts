type RawStyleConfig = {
	styleLibraryDirectory?: string;
	custom?: string;
	css?: string;
	bool?: boolean;
};

type RawPluginImportConfig = {
	libraryName: string;
	libraryDirectory?: string;
	customName?: string;
	customStyleName?: string;
	style?: RawStyleConfig;
	camelToDashComponentName?: boolean;
	transformToDefaultImport?: boolean;
	ignoreEsComponent?: Array<string>;
	ignoreStyleComponent?: Array<string>;
};

type PluginImportConfig = {
	libraryName: string;
	libraryDirectory?: string;
	customName?: string;
	customStyleName?: string;
	style?: string | boolean;
	styleLibraryDirectory?: string;
	camelToDashComponentName?: boolean;
	transformToDefaultImport?: boolean;
	ignoreEsComponent?: string[];
	ignoreStyleComponent?: string[];
};

type PluginImportOptions = PluginImportConfig[] | undefined;

function isObject(val: any): boolean {
	return Object.prototype.toString.call(val) === "[object Object]";
}

function resolvePluginImport(
	pluginImport: PluginImportOptions
): RawPluginImportConfig[] | undefined {
	if (!pluginImport) {
		return undefined;
	}

	return pluginImport.map(config => {
		const rawConfig: RawPluginImportConfig = {
			...config,
			style: {} // As babel-plugin-import style config is very flexible, we convert it to a more specific structure
		};

		if (typeof config.style === "boolean") {
			rawConfig.style!.bool = config.style;
		} else if (typeof config.style === "string") {
			const isTpl = config.style.includes("{{");
			rawConfig.style![isTpl ? "custom" : "css"] = config.style;
		} else if (isObject(config.style)) {
			// for child compiler
			// see https://github.com/web-infra-dev/rspack/issues/4597
			rawConfig.style = config.style;
		}

		// This option will overrides the behavior of style
		if (config.styleLibraryDirectory) {
			rawConfig.style = { styleLibraryDirectory: config.styleLibraryDirectory };
		}

		return rawConfig;
	});
}

export { resolvePluginImport };
export type { PluginImportOptions };
