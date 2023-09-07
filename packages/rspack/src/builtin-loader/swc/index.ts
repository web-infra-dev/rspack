import { resolveEmotion } from "./emotion";
import type { EmotionOptions } from "./emotion";

import { resolveReact } from "./react";
import type { ReactOptions } from "./react";

import { resolveRelay } from "./relay";
import type { RelayOptions } from "./relay";

import { resolvePluginImport } from "./pluginImport";
import type { PluginImportOptions } from "./pluginImport";

class RspackExperimentsBuilder {
	react?: ReturnType<typeof resolveReact>;
	import?: ReturnType<typeof resolvePluginImport>;
	emotion?: ReturnType<typeof resolveEmotion>;
	relay?: ReturnType<typeof resolveRelay>;

	static create() {
		return new this();
	}

	useReact(options: ReactOptions = {}) {
		this.react ??= resolveReact(options);
		return this;
	}

	useRelay(rootDir: string, options: RelayOptions = true) {
		this.relay ??= resolveRelay(options, rootDir);
		return this;
	}

	usePluginImport(options?: PluginImportOptions) {
		this.import ??= resolvePluginImport(options);
		return this;
	}

	useEmotion(isProduction: boolean, options: EmotionOptions = true) {
		this.emotion ??= resolveEmotion(options, isProduction);
		return this;
	}
}

export const createSwcLoaderExperiments = RspackExperimentsBuilder.create.bind(
	RspackExperimentsBuilder
);
export { resolveEmotion, resolveReact, resolveRelay, resolvePluginImport };
export type { EmotionOptions, ReactOptions, RelayOptions, PluginImportOptions };
