import { RawPluginImportConfig } from "@rspack/binding";

type PluginImportConfig = {
	libraryName: string;
	libraryDirectory?: string;
	customName?: string;
	customStyleName?: string;
	style?: string | boolean;
	styleLibraryDirectory?: string;
	camelToDashComponentName?: boolean;
	transformToDefaultImport?: boolean;
	ignoreEsComponent?: Array<string>;
	ignoreStyleComponent?: Array<string>;
};

type PluginImportOptions = PluginImportConfig[] | undefined;

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
