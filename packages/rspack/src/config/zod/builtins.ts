import { z } from "zod";

const css = z
	.strictObject({
		modules: z
			.strictObject({
				localsConvention: z.enum([
					"asIs",
					"camelCase",
					"camelCaseOnly",
					"dashes",
					"dashesOnly"
				]),
				localIdentName: z.string(),
				exportsOnly: z.boolean()
			})
			.partial()
	})
	.optional();

const postcss = z
	.strictObject({
		pxtorem: z
			.strictObject({
				rootValue: z.number().optional(),
				unitPrecision: z.number().optional(),
				selectorBlackList: z.string().array().optional(),
				propList: z.string().array().optional(),
				replace: z.boolean().optional(),
				mediaQuery: z.boolean().optional(),
				minPixelValue: z.number().optional()
			})
			.optional()
	})
	.refine(() => {
		console.warn(
			"warn: `builtins.postcss` is going to be deprecated and will be removed at 0.3. See details at https://github.com/web-infra-dev/rspack/issues/3452"
		);
		return true;
	})
	.optional();

const treeShaking = z.boolean().or(z.literal("module")).optional();

const progress = z
	.boolean()
	.or(
		z.strictObject({
			prefix: z.string().optional()
		})
	)
	.optional();

const react = z
	.strictObject({
		runtime: z.enum(["automatic", "classic"]).optional(),
		importSource: z.string().optional(),
		pragma: z.string().optional(),
		pragmaFrag: z.string().optional(),
		throwIfNamespace: z.boolean().optional(),
		development: z.boolean().optional(),
		useBuiltins: z.boolean().optional(),
		useSpread: z.boolean().optional(),
		refresh: z.boolean().optional()
	})
	.optional();

const noEmitAssets = z.boolean().optional();

const define = z
	.record(z.string(), z.string().or(z.boolean()).or(z.undefined()))
	.optional();

const provide = z
	.record(z.string(), z.string().or(z.string().array()))
	.optional();

const html = z
	.object({
		title: z.string().optional(),
		filename: z.string().optional(),
		template: z.string().optional(),
		templateParameters: z.record(z.string()).optional(),
		inject: z.enum(["head", "body"]).optional(),
		publicPath: z.string().optional(),
		scriptLoading: z.enum(["blocking", "defer", "module"]).optional(),
		chunks: z.string().array().optional(),
		excludedChunks: z.string().array().optional(),
		sri: z.enum(["sha256", "sha384", "sha512"]).optional(),
		minify: z.boolean().optional(),
		favicon: z.string().optional(),
		meta: z.record(z.string().or(z.record(z.string()))).optional()
	})
	.array()
	.optional();

const decorator = z
	.boolean()
	.or(
		z
			.strictObject({
				legacy: z.boolean(),
				emitMetadata: z.boolean()
			})
			.partial()
	)
	.optional();

const minifyCondition = z.string().or(z.instanceof(RegExp));

const minifyConditions = minifyCondition.or(minifyCondition.array());

const minifyOptions = z
	.strictObject({
		passes: z.number().optional(),
		dropConsole: z.boolean().optional(),
		pureFuncs: z.string().array().optional(),
		extractComments: z.boolean().or(z.instanceof(RegExp)).optional(),
		test: minifyConditions.optional(),
		exclude: minifyConditions.optional(),
		include: minifyConditions.optional()
	})
	.optional();

const emotion = z
	.boolean()
	.or(
		z.strictObject({
			sourceMap: z.boolean().optional(),
			autoLabel: z.enum(["never", "dev-only", "always"]).optional(),
			labelFormat: z.string().optional(),
			importMap: z
				.record(z.record(z.tuple([z.string(), z.string()])))
				.optional()
		})
	)
	.optional();

const presetEnv = z
	.strictObject({
		targets: z.string().array(),
		mode: z.enum(["usage", "entry"]).optional(),
		coreJs: z.string().optional()
	})
	.partial()
	.optional();

const polyfill = z.boolean().optional();

const devFriendlySplitChunks = z.boolean().optional();

const copy = z
	.strictObject({
		patterns: z
			.string()
			.or(
				z.strictObject({
					from: z.string(),
					to: z.string().optional(),
					context: z.string().optional(),
					toType: z.string().optional(),
					noErrorOnMissing: z.boolean().optional(),
					force: z.boolean().optional(),
					priority: z.number().optional(),
					globOptions: z
						.strictObject({
							caseSensitiveMatch: z.boolean().optional(),
							dot: z.boolean().optional(),
							ignore: z.string().array().optional()
						})
						.optional()
				})
			)
			.array()
	})
	.optional();

const bannerCondition = z.string().or(z.instanceof(RegExp));

const bannerConditions = bannerCondition.or(bannerCondition.array());

const bannerConfig = z.string().or(
	z.strictObject({
		banner: z.string(),
		entryOnly: z.boolean().optional(),
		footer: z.boolean().optional(),
		raw: z.boolean().optional(),
		test: bannerConditions.optional(),
		exclude: bannerConditions.optional(),
		include: bannerConditions.optional()
	})
);

const bannerConfigs = bannerConfig.or(bannerConfig.array());

const banner = bannerConfigs.optional();

const pluginImport = z
	.strictObject({
		libraryName: z.string(),
		libraryDirectory: z.string().optional(),
		customName: z.string().optional(),
		customStyleName: z.string().optional(),
		style: z.union([z.boolean(), z.string()]).optional(),
		styleLibraryDirectory: z.string().optional(),
		camelToDashComponentName: z.boolean().optional(),
		transformToDefaultImport: z.boolean().optional(),
		ignoreEsComponent: z.string().array().optional(),
		ignoreStyleComponent: z.string().array().optional()
	})
	.array()
	.optional();

const relay = z
	.boolean()
	.or(
		z.strictObject({
			artifactDirectory: z.string().optional(),
			language: z.enum(["javascript", "typescript", "flow"])
		})
	)
	.optional();

const codeGeneration = z
	.strictObject({
		keepComments: z.boolean()
	})
	.partial()
	.optional();

export function builtins() {
	return z.strictObject({
		css,
		postcss,
		treeShaking,
		progress,
		react,
		noEmitAssets,
		define,
		provide,
		html,
		decorator,
		minifyOptions,
		emotion,
		presetEnv,
		polyfill,
		devFriendlySplitChunks,
		copy,
		banner,
		pluginImport,
		relay,
		codeGeneration
	});
}
