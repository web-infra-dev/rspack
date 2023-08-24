import {
	RawDecoratorOptions,
	RawPluginImportConfig,
	RawPresetEnv,
	RawReactOptions,
	RawRelayConfig
} from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";
import path from "path";
import { loadConfig } from "browserslist";

export const DecoratorOptionsPlugin = create<
	undefined | Partial<RawDecoratorOptions>,
	RawDecoratorOptions
>(BuiltinPluginKind.DecoratorOptions, decorator => {
	if (decorator === undefined) {
		decorator = {};
	}
	return Object.assign(
		{
			legacy: true,
			emitMetadata: true
		},
		decorator
	);
});

export type EmotionConfigImportMap = {
	[packageName: string]: {
		[exportName: string]: {
			canonicalImport?: [string, string];
		};
	};
};

export type EmotionConfig = {
	sourceMap?: boolean;
	autoLabel?: "never" | "dev-only" | "always";
	labelFormat?: string;
	importMap?: EmotionConfigImportMap;
	production: boolean;
};

export const EmotionPlugin = create<EmotionConfig, string>(
	BuiltinPluginKind.Emotion,
	emotion => {
		const autoLabel = emotion.autoLabel ?? "dev-only";

		const emotionConfig = {
			enabled: true,
			autoLabel:
				autoLabel === "dev-only" ? !emotion.production : autoLabel === "always",
			importMap: emotion.importMap,
			labelFormat: emotion.labelFormat ?? "[local]",
			sourcemap: emotion.production ? false : emotion.sourceMap ?? true
		};

		return JSON.stringify(emotionConfig);
	}
);

function getRelayConfigFromProject(
	rootDir: string
): RawRelayConfig | undefined {
	for (const configName of [
		"relay.config.json",
		"relay.config.js",
		"package.json"
	]) {
		const configPath = path.join(rootDir, configName);
		try {
			let config = require(configPath) as
				| Partial<RawRelayConfig>
				| { relay?: Partial<RawRelayConfig> }
				| undefined;

			let finalConfig: Partial<RawRelayConfig> | undefined;
			if (configName === "package.json") {
				finalConfig = (config as { relay?: Partial<RawRelayConfig> })?.relay;
			} else {
				finalConfig = config as Partial<RawRelayConfig> | undefined;
			}

			if (finalConfig) {
				return {
					language: finalConfig.language!,
					artifactDirectory: finalConfig.artifactDirectory
				};
			}
		} catch (_) {}
	}
}
export const RelayPlugin = create<
	{ context: string } & RawRelayConfig,
	RawRelayConfig
>(BuiltinPluginKind.Relay, relay => {
	// Search relay config based on
	if ("context" in relay) {
		return (
			getRelayConfigFromProject(relay.context) || {
				language: "javascript"
			}
		);
	} else {
		return relay;
	}
});

export const PresetEnvPlugin = create<
	{ context: string } & Partial<RawPresetEnv>,
	RawPresetEnv
>(BuiltinPluginKind.PresetEnv, presetEnv => {
	return {
		targets:
			presetEnv?.targets ?? loadConfig({ path: presetEnv.context }) ?? [],
		mode: presetEnv?.mode,
		coreJs: presetEnv?.coreJs
	};
});

export const ReactOptionsPlugin = create<
	undefined | RawReactOptions,
	RawReactOptions
>(BuiltinPluginKind.ReactOptions, (options = {}) => options);

export type PluginImportConfig = {
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

export const PluginImportPlugin = create<
	PluginImportConfig[],
	RawPluginImportConfig[]
>(BuiltinPluginKind.PluginImport, pluginImport => {
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
});
