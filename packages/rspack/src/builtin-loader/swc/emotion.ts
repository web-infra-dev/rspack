type EmotionConfigImportMap = {
	[packageName: string]: {
		[exportName: string]: {
			canonicalImport?: [string, string];
		};
	};
};

type EmotionConfig = {
	sourceMap?: boolean;
	autoLabel?: "never" | "dev-only" | "always";
	labelFormat?: string;
	importMap?: EmotionConfigImportMap;
};

type EmotionOptions = boolean | EmotionConfig | undefined;

function resolveEmotion(
	emotion: EmotionOptions,
	isProduction: boolean
): EmotionConfig | undefined {
	if (!emotion) {
		return undefined;
	}

	if (emotion === true) {
		emotion = {};
	}

	const autoLabel = emotion?.autoLabel ?? "dev-only";

	const emotionConfig: EmotionConfig = {
		enabled: true,
		// @ts-expect-error autoLabel is string for JavaScript interface, however is boolean for Rust interface
		autoLabel:
			autoLabel === "dev-only" ? !isProduction : autoLabel === "always",
		importMap: emotion?.importMap,
		labelFormat: emotion?.labelFormat ?? "[local]",
		sourcemap: isProduction ? false : emotion?.sourceMap ?? true
	};

	return emotionConfig;
}

export { resolveEmotion };
export type { EmotionOptions };
