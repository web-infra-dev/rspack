import type {
	Config,
	EnvConfig,
	EsParserConfig,
	JscConfig,
	ModuleConfig,
	ParserConfig,
	TransformConfig,
	TsParserConfig
} from "@swc/types";
import type { PluginImportOptions } from "./pluginImport";

export type SwcLoaderEnvConfig = EnvConfig;
export type SwcLoaderJscConfig = JscConfig;
export type SwcLoaderModuleConfig = ModuleConfig;
export type SwcLoaderParserConfig = ParserConfig;
export type SwcLoaderEsParserConfig = EsParserConfig;
export type SwcLoaderTsParserConfig = TsParserConfig;
export type SwcLoaderTransformConfig = TransformConfig;
export type SwcLoaderOptions = Config & {
	isModule?: boolean | "unknown";
	/**
	 * Experimental features provided by Rspack.
	 * @experimental
	 */
	rspackExperiments?: {
		import?: PluginImportOptions;
	};
};
