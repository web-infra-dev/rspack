import { RawDecoratorOptions, RawRelayConfig } from "@rspack/binding";
import { BuiltinPluginKind, create } from "./base";
import path from "path";

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
	{ context: string } | RawRelayConfig,
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
