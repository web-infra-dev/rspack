import type { RawRelayConfig } from "@rspack/binding";
import path from "path";

type RelayOptions = boolean | RawRelayConfig | undefined;

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

function resolveRelay(
	relay: RelayOptions,
	rootDir: string
): RawRelayConfig | undefined {
	if (!relay) {
		return undefined;
	}

	// Search relay config based on
	if (relay === true) {
		return (
			getRelayConfigFromProject(rootDir) || {
				language: "javascript"
			}
		);
	} else {
		return relay;
	}
}

export { resolveRelay };
export type { RelayOptions };
