/**
 * The following code is modified based on
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/index.js
 *
 * MIT Licensed
 * Author Jan Nicklas
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/LICENSE
 */
import type { Compiler, Compilation, RspackPluginInstance } from "@rspack/core";
// @ts-ignore
import type { Options as MinifyOptions } from "html-minifier-terser";
import assert from "assert";
import path from "path";
import fs from "fs";
import chunkSorter from "./chunkSorter";
import * as template from "./template";
import { getHtmlRspackPluginHooks } from "./hooks";
import {
	HtmlTagArray,
	htmlTagObjectToString,
	createHtmlTagObject
} from "./html-tags";

export type { HtmlRspackPluginHooks } from "./hooks";
import { defaultTemplateCompiler } from "./template";

export interface Options {
	/**
	 * Emit the file only if it was changed.
	 * @default true
	 */
	cache?: boolean;
	/**
	 * List all entries which should be injected
	 */
	chunks?: "all" | string[];
	/**
	 * Allows to control how chunks should be sorted before they are included to the html.
	 * @default 'auto'
	 */
	chunksSortMode?:
		| "auto"
		| "manual"
		| ((entryNameA: string, entryNameB: string) => number);
	/**
	 * List all entries which should not be injected
	 */
	excludeChunks?: string[];
	/**
	 * Path to the favicon icon
	 */
	favicon?: false | string;
	/**
	 * The file to write the HTML to.
	 * Supports subdirectories eg: `assets/admin.html`
	 * [name] will be replaced by the entry name
	 * Supports a function to generate the name
	 *
	 * @default 'index.html'
	 */
	filename?: string | ((entryName: string) => string);
	/**
	 * By default the public path is set to `auto` - that way the html-webpack-plugin will try
	 * to set the publicPath according to the current filename and the webpack publicPath setting
	 */
	publicPath?: string | "auto";
	/**
	 * If `true` then append a unique `webpack` compilation hash to all included scripts and CSS files.
	 * This is useful for cache busting
	 */
	hash?: boolean;
	/**
	 * Inject all assets into the given `template` or `templateContent`.
	 */
	inject?:
		| false // Don't inject scripts
		| true // Inject scripts into body
		| "body" // Inject scripts into body
		| "head"; // Inject scripts into head
	/**
	 * Set up script loading
	 * blocking will result in <script src="..."></script>
	 * defer will result in <script defer src="..."></script>
	 *
	 * @default 'defer'
	 */
	scriptLoading?: "blocking" | "defer" | "module";
	/**
	 * Inject meta tags
	 */
	meta?:
		| false // Disable injection
		| {
				[name: string]:
					| string
					| false // name content pair e.g. {viewport: 'width=device-width, initial-scale=1, shrink-to-fit=no'}`
					| { [attributeName: string]: string | boolean }; // custom properties e.g. { name:"viewport" content:"width=500, initial-scale=1" }
		  };
	/**
	 * HTML Minification options accepts the following values:
	 * - Set to `false` to disable minification
	 * - Set to `'auto'` to enable minification only for production mode
	 * - Set to custom minification according to
	 * {@link https://github.com/kangax/html-minifier#options-quick-reference}
	 */
	minify?: "auto" | boolean | MinifyOptions;
	/**
	 * Render errors into the HTML page
	 */
	showErrors?: boolean;
	/**
	 * The `webpack` require path to the template.
	 * @see https://github.com/jantimon/html-webpack-plugin/blob/master/docs/template-option.md
	 */
	template?: string;
	/**
	 * Allow to use a html string instead of reading from a file
	 */
	templateContent?:
		| false // Use the template option instead to load a file
		| string
		| ((templateParameters: {
				[option: string]: any;
		  }) => string | Promise<string>)
		| Promise<string>;
	/**
	 * Compile template to js code.
	 */
	templateCompiler?: template.TemplateCompiler;
	/**
	 * Allows to overwrite the parameters used in the template
	 */
	templateParameters?:
		| false // Pass an empty object to the template function
		| ((
				compilation: any,
				assets: {
					publicPath: string;
					js: Array<string>;
					css: Array<string>;
					manifest?: string;
					favicon?: string;
				},
				assetTags: {
					headTags: HtmlTagObject[];
					bodyTags: HtmlTagObject[];
				},
				options: ProcessedOptions
		  ) => { [option: string]: any } | Promise<{ [option: string]: any }>)
		| { [option: string]: any };
	/**
	 * The title to use for the generated HTML document
	 */
	title?: string;
	/**
	 * Enforce self closing tags e.g. <link />
	 */
	xhtml?: boolean;
	/**
	 * In addition to the options actually used by this plugin, you can use this hash to pass arbitrary data through
	 * to your template.
	 */
	[option: string]: any;
}

/**
 * The plugin options after adding default values
 */
export interface ProcessedOptions extends Required<Options> {
	filename: string;
}

interface TemplateParameters {
	[option: string]: any;
}

type TemplateFunction = (
	params: TemplateParameters
) => string | Promise<string>;

/**
 * A tag element according to the HtmlRspackPlugin object notation
 */
export interface HtmlTagObject {
	/**
	 * Attributes of the html tag
	 * E.g. `{'disabled': true, 'value': 'demo'}`
	 */
	attributes: {
		[attributeName: string]: string | boolean | null | undefined;
	};
	/**
	 * The tag name e.g. `'div'`
	 */
	tagName: string;
	/**
	 * The inner HTML
	 */
	innerHTML?: string;
	/**
	 * Whether this html must not contain innerHTML
	 * @see https://www.w3.org/TR/html5/syntax.html#void-elements
	 */
	voidTag: boolean;
	/**
	 * Meta information about the tag
	 * E.g. `{'plugin': 'html-webpack-plugin'}`
	 */
	meta: {
		plugin?: string;
		[metaAttributeName: string]: any;
	};
}

export interface AssetTags {
	headTags: HtmlTagObject[];
	bodyTags: HtmlTagObject[];
}

export interface Assets {
	publicPath: string;
	js: string[];
	css: string[];
	manifest?: string;
	favicon?: string;
}

/**
 * The values which are available during template execution
 *
 * Please keep in mind that the `templateParameter` options allows to change them
 */
export interface TemplateParameter {
	compilation: Compilation;
	htmlWebpackPlugin: {
		tags: {
			headTags: HtmlTagObject[];
			bodyTags: HtmlTagObject[];
		};
		files: {
			publicPath: string;
			js: Array<string>;
			css: Array<string>;
			manifest?: string;
			favicon?: string;
		};
		options: Options;
	};
	webpackConfig: any;
}

export default class HtmlRspackPlugin implements RspackPluginInstance {
	name = "HtmlRspackPlugin";

	userOptions: Options;
	options?: ProcessedOptions;

	constructor(options?: Options) {
		this.userOptions = options || {};
	}
	static defaultTemplateCompiler = defaultTemplateCompiler;
	static getHooks = getHtmlRspackPluginHooks;

	static createHtmlTagObject = createHtmlTagObject;

	apply(compiler: Compiler) {
		// Wait for configuration preset plugions to apply all configure webpack defaults
		compiler.hooks.initialize.tap("HtmlRspackPlugin", () => {
			const userOptions = this.userOptions;

			// Default options
			const defaultOptions: ProcessedOptions = {
				template: "auto",
				templateContent: false,
				templateCompiler: template.defaultTemplateCompiler,
				templateParameters: templateParametersGenerator,
				filename: "index.html",
				publicPath:
					userOptions.publicPath === undefined
						? "auto"
						: userOptions.publicPath,
				hash: false,
				inject: userOptions.scriptLoading === "blocking" ? "body" : "head",
				scriptLoading: "defer",
				compile: true,
				favicon: false,
				minify: "auto",
				cache: true,
				showErrors: true,
				chunks: "all",
				excludeChunks: [],
				chunksSortMode: "auto",
				meta: {},
				base: false,
				title: "Webpack App",
				xhtml: false
			};

			const options: ProcessedOptions = Object.assign(
				defaultOptions,
				userOptions
			);
			this.options = options;

			// Assert correct option spelling
			assert(
				options.scriptLoading === "defer" ||
					options.scriptLoading === "blocking" ||
					options.scriptLoading === "module",
				'scriptLoading needs to be set to "defer", "blocking" or "module"'
			);
			assert(
				options.inject === true ||
					options.inject === false ||
					options.inject === "head" ||
					options.inject === "body",
				'inject needs to be set to true, false, "head" or "body'
			);

			// Default metaOptions if no template is provided
			if (
				!userOptions.template &&
				options.templateContent === false &&
				options.meta
			) {
				const defaultMeta = {
					// From https://developer.mozilla.org/en-US/docs/Mozilla/Mobile/Viewport_meta_tag
					viewport: "width=device-width, initial-scale=1"
				};
				options.meta = Object.assign(
					{},
					options.meta,
					defaultMeta,
					userOptions.meta
				);
			}

			// entryName to fileName conversion function
			const userOptionFilename =
				userOptions.filename || defaultOptions.filename;
			const filenameFunction =
				typeof userOptionFilename === "function"
					? userOptionFilename
					: // Replace '[name]' with entry name
						(entryName: string) =>
							userOptionFilename.replace(/\[name\]/g, entryName);

			/** output filenames for the given entry names */
			const entryNames = Object.keys(compiler.options.entry);
			const outputFileNames = new Set(
				(entryNames.length ? entryNames : ["main"]).map(filenameFunction)
			);

			/** Option for every entry point */
			const entryOptions = Array.from(outputFileNames).map(filename => ({
				...options,
				filename
			}));

			// Hook all options into the webpack compiler
			entryOptions.forEach(instanceOptions => {
				hookIntoCompiler(compiler, instanceOptions, this);
			});
		});
	}
}

function templateParametersGenerator(
	compilation: Compilation,
	assets: Assets,
	assetTags: AssetTags,
	options: ProcessedOptions
): TemplateParameter {
	return {
		compilation: compilation,
		webpackConfig: compilation.options,
		htmlWebpackPlugin: {
			tags: assetTags,
			files: assets,
			options: options
		}
	};
}

function hookIntoCompiler(
	compiler: Compiler,
	options: ProcessedOptions,
	plugin: HtmlRspackPlugin
) {
	const webpack = compiler.webpack;
	options.template = getFullTemplatePath(options.template, compiler.context);

	// convert absolute filename into relative so that webpack can
	// generate it at correct location
	const filename = options.filename;
	if (path.resolve(filename) === path.normalize(filename)) {
		// options.output.path must have value after `applyRspackOptionsDefaults`
		const outputPath = compiler.options.output.path!;
		options.filename = path.relative(outputPath, filename);
	}

	// Check if webpack is running in production mode
	// @see https://github.com/webpack/webpack/blob/3366421f1784c449f415cda5930a8e445086f688/lib/WebpackOptionsDefaulter.js#L12-L14
	const isProductionLikeMode =
		compiler.options.mode === "production" || !compiler.options.mode;

	const minify = options.minify;
	if (minify === true || (minify === "auto" && isProductionLikeMode)) {
		options.minify = {
			// https://www.npmjs.com/package/html-minifier-terser#options-quick-reference
			collapseWhitespace: true,
			keepClosingSlash: true,
			removeComments: true,
			removeRedundantAttributes: true,
			removeScriptTypeAttributes: true,
			removeStyleLinkTypeAttributes: true,
			useShortDoctype: true
		};
	}

	compiler.hooks.thisCompilation.tap("HtmlRspackPlugin", compilation => {
		compilation.hooks.processAssets.tapAsync(
			{
				name: "HtmlRspackPlugin",
				stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE
			},
			(compilationAssets, callback) => {
				// Get all entry point names for this html file
				const entryNames = Array.from(compilation.entrypoints.keys());
				const filteredEntryNames = filterChunks(
					entryNames,
					options.chunks,
					options.excludeChunks
				);
				const sortedEntryNames = sortEntryChunks(
					filteredEntryNames,
					options.chunksSortMode,
					compilation
				);

				/** The public path used inside the html file */
				const htmlPublicPath = getPublicPath(
					compilation,
					options.filename,
					options.publicPath
				);

				/** Generated file paths from the entry point names */
				const assets = htmlRspackPluginAssets(
					compilation,
					sortedEntryNames,
					htmlPublicPath
				);

				// The html-webpack plugin uses a object representation for the html-tags which will be injected
				// to allow altering them more easily
				// Just before they are converted a third-party-plugin author might change the order and content
				const assetsPromise = getFaviconPublicPath(
					options.favicon,
					compilation,
					assets.publicPath
				).then(faviconPath => {
					assets.favicon = faviconPath;
					return getHtmlRspackPluginHooks(
						compilation
					).beforeAssetTagGeneration.promise({
						assets: assets,
						outputName: options.filename,
						plugin: plugin
					});
				});

				// Turn the js and css paths into grouped HtmlTagObjects
				const assetTagGroupsPromise = assetsPromise
					// And allow third-party-plugin authors to reorder and change the assetTags before they are grouped
					.then(({ assets }) =>
						getHtmlRspackPluginHooks(compilation).alterAssetTags.promise({
							assetTags: {
								scripts: generatedScriptTags(assets.js),
								styles: generateStyleTags(assets.css),
								meta: [
									...generateBaseTag(options.base),
									...generatedMetaTags(options.meta),
									...generateFaviconTags(assets.favicon)
								]
							},
							outputName: options.filename,
							publicPath: htmlPublicPath,
							plugin: plugin
						})
					)
					.then(({ assetTags }) => {
						// Inject scripts to body unless it set explicitly to head
						const scriptTarget =
							options.inject === "head" ||
							(options.inject !== "body" &&
								options.scriptLoading !== "blocking")
								? "head"
								: "body";
						// Group assets to `head` and `body` tag arrays
						const assetGroups = generateAssetGroups(assetTags, scriptTarget);
						// Allow third-party-plugin authors to reorder and change the assetTags once they are grouped
						return getHtmlRspackPluginHooks(
							compilation
						).alterAssetTagGroups.promise({
							headTags: assetGroups.headTags,
							bodyTags: assetGroups.bodyTags,
							outputName: options.filename,
							publicPath: htmlPublicPath,
							plugin: plugin
						});
					});

				// Turn the compiled template into a nodejs function or into a nodejs string
				const templateEvaluationPromise = Promise.resolve().then(() => {
					// Allow to use a custom function / string instead
					if (options.templateContent !== false) {
						return options.templateContent;
					}
					// Once everything is compiled evaluate the html factory
					// and replace it with its content
					const compileOptions = options.templateCompiler.options;
					return fs.promises
						.readFile(options.template, "utf-8")
						.then(content =>
							options.templateCompiler.compile(content, {
								filename: options.template,
								...compileOptions
							})
						)
						.then(compiled =>
							template.evaluate(compiled, htmlPublicPath, options.template)
						);
				});

				const templateExecutionPromise = Promise.all([
					assetsPromise,
					assetTagGroupsPromise,
					templateEvaluationPromise
				])
					// Execute the template
					.then(([assetsHookResult, assetTags, compilationResult]) =>
						typeof compilationResult !== "function"
							? compilationResult
							: executeTemplate(
									compilationResult,
									assetsHookResult.assets,
									{
										headTags: assetTags.headTags,
										bodyTags: assetTags.bodyTags
									},
									compilation
								)
					);

				const injectedHtmlPromise = Promise.all([
					assetTagGroupsPromise,
					templateExecutionPromise
				])
					// Allow plugins to change the html before assets are injected
					.then(([assetTags, html]) => {
						const pluginArgs = {
							html,
							headTags: assetTags.headTags,
							bodyTags: assetTags.bodyTags,
							plugin: plugin,
							outputName: options.filename
						};
						return getHtmlRspackPluginHooks(
							compilation
						).afterTemplateExecution.promise(pluginArgs);
					})
					.then(({ html, headTags, bodyTags }) => {
						return postProcessHtml(html, assets, { headTags, bodyTags });
					});

				const emitHtmlPromise = injectedHtmlPromise
					// Allow plugins to change the html after assets are injected
					.then(html => {
						const pluginArgs = {
							html,
							plugin: plugin,
							outputName: options.filename
						};
						return getHtmlRspackPluginHooks(compilation)
							.beforeEmit.promise(pluginArgs)
							.then(result => result.html);
					})
					.catch(err => {
						// In case anything went wrong the promise is resolved
						// with the error message and an error is logged
						compilation.errors.push(err);
						return `ERROR: ${err.message}\n${err.stack}`;
					})
					.then(html => {
						// TODO:
						// const replacedFilename = replacePlaceholdersInFilename(options.filename, html, compilation);
						// Add the evaluated html code to the webpack assets
						compilation.emitAsset(
							options.filename,
							new webpack.sources.RawSource(html, false)
						);
						return options.filename;
					})
					.then(finalOutputName =>
						getHtmlRspackPluginHooks(compilation)
							.afterEmit.promise({
								outputName: finalOutputName,
								plugin: plugin
							})
							.catch(err => {
								console.error(err);
								return null;
							})
							.then(() => null)
					);

				// Once all files are added to the webpack compilation
				// let the webpack compiler continue
				emitHtmlPromise.then(() => {
					callback();
				});
			}
		);
	});

	function getPublicPath(
		compilation: Compilation,
		filename: string,
		customPublicPath: string | "auto"
	): string {
		// compilation should have hash/publicPath
		const compilationHash = compilation.hash!;

		const rspackPublicPath = compilation.getAssetPath(
			compilation.outputOptions.publicPath!,
			{ hash: compilationHash }
		);

		// Webpack 5 introduced "auto" as default value
		const isPublicPathDefined = rspackPublicPath !== "auto";

		const outputPath = compilation.options.output.path!;
		let publicPath =
			// If the html-webpack-plugin options contain a custom public path uset it
			customPublicPath !== "auto"
				? customPublicPath
				: isPublicPathDefined
					? // If a hard coded public path exists use it
						rspackPublicPath
					: // If no public path was set get a relative url path
						path
							.relative(
								path.resolve(outputPath, path.dirname(filename)),
								outputPath
							)
							.split(path.sep)
							.join("/");

		if (publicPath.length && publicPath.substr(-1, 1) !== "/") {
			publicPath += "/";
		}

		return publicPath;
	}

	function htmlRspackPluginAssets(
		compilation: Compilation,
		entryNames: string[],
		publicPath: string | "auto"
	): Assets {
		const compilationHash = compilation.hash!;
		const assets: Assets = {
			// The public path
			publicPath,
			// Will contain all js and mjs files
			js: [],
			// Will contain all css files
			css: [],
			// Will contain the html5 appcache manifest files if it exists
			manifest: Object.keys(compilation.assets).find(
				assetFile => path.extname(assetFile) === ".appcache"
			),
			// Favicon
			favicon: undefined
		};

		// Append a hash for cache busting
		if (options.hash && assets.manifest) {
			assets.manifest = appendHash(assets.manifest, compilationHash);
		}

		// Extract paths to .js, .mjs and .css files from the current compilation
		const entryPointPublicPathMap = {} as Record<string, boolean>;
		const extensionRegexp = /\.(css|js|mjs)(\?|$)/;
		for (let i = 0; i < entryNames.length; i++) {
			const entryName = entryNames[i];
			/** entryPointUnfilteredFiles - also includes hot module update files */
			const entryPointUnfilteredFiles = compilation.entrypoints
				.get(entryName)!
				.getFiles();

			const entryPointFiles = entryPointUnfilteredFiles.filter(chunkFile => {
				const asset = compilation.getAsset(chunkFile);
				if (!asset) {
					return true;
				}
				// Prevent hot-module files from being included:
				const assetMetaInformation = asset.info;
				return !(
					assetMetaInformation.hotModuleReplacement ||
					assetMetaInformation.development
				);
			});

			// Prepend the publicPath and append the hash depending on the
			// rspack.output.publicPath and hashOptions
			// E.g. bundle.js -> /bundle.js?hash
			const entryPointPublicPaths = entryPointFiles.map(chunkFile => {
				const entryPointPublicPath = publicPath + urlencodePath(chunkFile);
				return options.hash
					? appendHash(entryPointPublicPath, compilationHash)
					: entryPointPublicPath;
			});

			entryPointPublicPaths.forEach(entryPointPublicPath => {
				const extMatch = extensionRegexp.exec(entryPointPublicPath);
				// Skip if the public path is not a .css, .mjs or .js file
				if (!extMatch) {
					return;
				}
				// Skip if this file is already known
				// (e.g. because of common chunk optimizations)
				if (entryPointPublicPathMap[entryPointPublicPath]) {
					return;
				}
				entryPointPublicPathMap[entryPointPublicPath] = true;
				// ext will contain .js or .css, because .mjs recognizes as .js
				const ext =
					extMatch[1] === "mjs" ? "js" : (extMatch[1] as "js" | "css");
				assets[ext].push(entryPointPublicPath);
			});
		}

		return assets;
	}

	function appendHash(url: string, hash: string) {
		if (!url) {
			return url;
		}
		return url + (url.indexOf("?") === -1 ? "?" : "&") + hash;
	}

	function sortEntryChunks(
		entryNames: string[],
		sortMode:
			| "auto"
			| "manual"
			| ((entryNameA: string, entryNameB: string) => number),
		compilation: Compilation
	): string[] {
		// Custom function
		if (typeof sortMode === "function") {
			return entryNames.sort(sortMode);
		} else if (typeof chunkSorter[sortMode] !== "undefined") {
			// Check if the given sort mode is a valid chunkSorter sort mode
			// N.B. sortMode is a user input and may be unsafe to call object method
			return chunkSorter[sortMode](entryNames, compilation, options);
		}
		throw new Error('"' + sortMode + '" is not a valid chunk sort mode');
	}

	function filterChunks(
		chunks: string[],
		includedChunks: string[] | "all",
		excludedChunks: string[]
	) {
		return chunks.filter(chunkName => {
			// Skip if the chunks should be filtered and the given chunk was not added explicitly
			if (
				Array.isArray(includedChunks) &&
				includedChunks.indexOf(chunkName) === -1
			) {
				return false;
			}
			// Skip if the chunks should be filtered and the given chunk was excluded explicitly
			if (
				Array.isArray(excludedChunks) &&
				excludedChunks.indexOf(chunkName) !== -1
			) {
				return false;
			}
			// Add otherwise
			return true;
		});
	}

	/**
	 * Encode each path component using `encodeURIComponent` as files can contain characters
	 * which needs special encoding in URLs like `+ `.
	 *
	 * Valid filesystem characters which need to be encoded for urls:
	 *
	 * # pound, % percent, & ampersand, { left curly bracket, } right curly bracket,
	 * \ back slash, < left angle bracket, > right angle bracket, * asterisk, ? question mark,
	 * blank spaces, $ dollar sign, ! exclamation point, ' single quotes, " double quotes,
	 * : colon, @ at sign, + plus sign, ` backtick, | pipe, = equal sign
	 *
	 * However the query string must not be encoded:
	 *
	 *  fo:demonstration-path/very fancy+name.js?path=/home?value=abc&value=def#zzz
	 *    ^             ^    ^    ^     ^    ^  ^    ^^    ^     ^   ^     ^   ^
	 *    |             |    |    |     |    |  |    ||    |     |   |     |   |
	 *    encoded       |    |    encoded    |  |    ||    |     |   |     |   |
	 *                 ignored              ignored  ignored     ignored   ignored
	 *
	 */
	function urlencodePath(filePath: string) {
		// People use the filepath in quite unexpected ways.
		// Try to extract the first querystring of the url:
		//
		// some+path/demo.html?value=abc?def
		//
		const queryStringStart = filePath.indexOf("?");
		const urlPath =
			queryStringStart === -1 ? filePath : filePath.substr(0, queryStringStart);
		const queryString = filePath.substr(urlPath.length);
		// Encode all parts except '/' which are not part of the querystring:
		const encodedUrlPath = urlPath.split("/").map(encodeURIComponent).join("/");
		return encodedUrlPath + queryString;
	}

	/**
	 * Converts a favicon file from disk to a webpack resource
	 * and returns the url to the resource
	 */
	function getFaviconPublicPath(
		faviconFilePath: string | false,
		compilation: Compilation,
		publicPath: string
	) {
		if (!faviconFilePath) {
			return Promise.resolve(undefined);
		}
		return addFileToAssets(faviconFilePath, compilation).then(faviconName => {
			const faviconPath = publicPath + faviconName;
			if (options.hash) {
				return appendHash(faviconPath, compilation.hash!);
			}
			return faviconPath;
		});
	}

	function addFileToAssets(filename: string, compilation: Compilation) {
		filename = path.resolve(compilation.compiler.context, filename);
		return fs.promises
			.readFile(filename)
			.then(source => new webpack.sources.RawSource(source, false))
			.catch(() =>
				Promise.reject(
					new Error("HtmlRspackPlugin: could not load file " + filename)
				)
			)
			.then(rawSource => {
				const basename = path.basename(filename);
				// compilation.fileDependencies.add(filename);
				compilation.emitAsset(basename, rawSource);
				return basename;
			});
	}

	function getFullTemplatePath(template: string, context: string): string {
		if (template === "auto") {
			template = path.resolve(context, "src/index.ejs");
			if (!fs.existsSync(template)) {
				template = path.join(__dirname, "../default_index.ejs");
			}
		}
		return path.resolve(context, template);
	}

	/**
	 * Generate all tags script for the given file paths
	 */
	function generatedScriptTags(jsAssets: string[]): HtmlTagObject[] {
		return jsAssets.map(scriptAsset => ({
			tagName: "script",
			voidTag: false,
			meta: { plugin: "html-rspack-plugin" },
			attributes: {
				defer: options.scriptLoading === "defer",
				type: options.scriptLoading === "module" ? "module" : undefined,
				src: scriptAsset
			}
		}));
	}

	/**
	 * Generate all style tags for the given file paths
	 */
	function generateStyleTags(cssAssets: string[]): HtmlTagObject[] {
		return cssAssets.map(styleAsset => ({
			tagName: "link",
			voidTag: true,
			meta: { plugin: "html-rspack-plugin" },
			attributes: {
				href: styleAsset,
				rel: "stylesheet"
			}
		}));
	}

	/**
	 * Generate an optional base tag
	 */
	function generateBaseTag(
		baseOption: false | string | { [attr: string]: string }
	): HtmlTagObject[] {
		if (baseOption === false) {
			return [];
		} else {
			return [
				{
					tagName: "base",
					voidTag: true,
					meta: { plugin: "html-rspack-plugin" },
					attributes:
						typeof baseOption === "string"
							? {
									href: baseOption
								}
							: baseOption
				}
			];
		}
	}

	/**
	 * Generate all meta tags for the given meta configuration
	 */
	function generatedMetaTags(
		metaOptions:
			| false
			| {
					[name: string]: false | string | { [attr: string]: string | boolean };
			  }
	): HtmlTagObject[] {
		if (metaOptions === false) {
			return [];
		}
		// Make tags self-closing in case of xhtml
		// Turn { "viewport" : "width=500, initial-scale=1" } into
		// [{ name:"viewport" content:"width=500, initial-scale=1" }]
		const metaTagAttributeObjects = Object.keys(metaOptions)
			.map(metaName => {
				const metaTagContent = metaOptions[metaName];
				return typeof metaTagContent === "string"
					? {
							name: metaName,
							content: metaTagContent
						}
					: metaTagContent;
			})
			.filter(attribute => attribute !== false);
		// Turn [{ name:"viewport" content:"width=500, initial-scale=1" }] into
		// the html-webpack-plugin tag structure
		return metaTagAttributeObjects.map(metaTagAttributes => {
			if (metaTagAttributes === false) {
				throw new Error("Invalid meta tag");
			}
			return {
				tagName: "meta",
				voidTag: true,
				meta: { plugin: "html-rspack-plugin" },
				attributes: metaTagAttributes
			};
		});
	}

	/**
	 * Generate a favicon tag for the given file path
	 */
	function generateFaviconTags(
		faviconPath: string | undefined
	): HtmlTagObject[] {
		if (!faviconPath) {
			return [];
		}
		return [
			{
				tagName: "link",
				voidTag: true,
				meta: { plugin: "html-webpack-plugin" },
				attributes: {
					rel: "icon",
					href: faviconPath
				}
			}
		];
	}

	/**
	 * Group assets to head and bottom tags
	 */
	function generateAssetGroups(
		assetTags: {
			scripts: HtmlTagObject[];
			styles: HtmlTagObject[];
			meta: HtmlTagObject[];
		},
		scriptTarget: "body" | "head"
	): AssetTags {
		const result = {
			headTags: [...assetTags.meta, ...assetTags.styles],
			bodyTags: [] as HtmlTagObject[]
		};
		// Add script tags to head or body depending on
		// the htmlPluginOptions
		if (scriptTarget === "body") {
			result.bodyTags.push(...assetTags.scripts);
		} else {
			// If script loading is blocking add the scripts to the end of the head
			// If script loading is non-blocking add the scripts infront of the css files
			const insertPosition =
				options.scriptLoading === "blocking"
					? result.headTags.length
					: assetTags.meta.length;
			result.headTags.splice(insertPosition, 0, ...assetTags.scripts);
		}
		return result;
	}

	function executeTemplate(
		templateFunction: TemplateFunction,
		assets: Assets,
		assetTags: AssetTags,
		compilation: Compilation
	): Promise<string> {
		// Template processing
		const templateParamsPromise = getTemplateParameters(
			compilation,
			assets,
			assetTags
		);
		return templateParamsPromise.then(templateParams => {
			try {
				// If html is a promise return the promise
				// If html is a string turn it into a promise
				return templateFunction(templateParams);
			} catch (e) {
				compilation.errors.push(new Error("Template execution failed: " + e));
				return Promise.reject(e);
			}
		});
	}

	function getTemplateParameters(
		compilation: Compilation,
		assets: Assets,
		assetTags: AssetTags
	) {
		const templateParameters = options.templateParameters;
		if (templateParameters === false) {
			return Promise.resolve({});
		}
		if (
			typeof templateParameters !== "function" &&
			typeof templateParameters !== "object"
		) {
			throw new Error(
				"templateParameters has to be either a function or an object"
			);
		}
		const templateParameterFunction =
			typeof templateParameters === "function"
				? // A custom function can overwrite the entire template parameter preparation
					templateParameters
				: // If the template parameters is an object merge it with the default values
					(
						compilation: Compilation,
						assets: Assets,
						assetTags: AssetTags,
						options: ProcessedOptions
					) =>
						Object.assign(
							{},
							templateParametersGenerator(
								compilation,
								assets,
								assetTags,
								options
							),
							templateParameters
						);
		const preparedAssetTags = {
			headTags: prepareAssetTagGroupForRendering(assetTags.headTags),
			bodyTags: prepareAssetTagGroupForRendering(assetTags.bodyTags)
		};
		return Promise.resolve().then(() =>
			templateParameterFunction(compilation, assets, preparedAssetTags, options)
		);
	}

	/**
	 * Add toString methods for easier rendering
	 * inside the template
	 */
	function prepareAssetTagGroupForRendering(
		assetTagGroup: HtmlTagObject[]
	): HtmlTagObject[] {
		const xhtml = options.xhtml;
		return HtmlTagArray.from(
			assetTagGroup.map(assetTag => {
				const copiedAssetTag = Object.assign({}, assetTag);
				copiedAssetTag.toString = function () {
					return htmlTagObjectToString(this, xhtml);
				};
				return copiedAssetTag;
			})
		);
	}

	/**
	 * Html Post processing
	 */
	function postProcessHtml(
		html: unknown,
		assets: Assets,
		assetTags: AssetTags
	): Promise<string> {
		if (typeof html !== "string") {
			return Promise.reject(
				new Error(
					"Expected html to be a string but got " + JSON.stringify(html)
				)
			);
		}
		const htmlAfterInjection = options.inject
			? injectAssetsIntoHtml(html, assets, assetTags)
			: html;
		const htmlAfterMinification = minifyHtml(htmlAfterInjection);
		return Promise.resolve(htmlAfterMinification);
	}

	/**
	 * Injects the assets into the given html string
	 */
	function injectAssetsIntoHtml(
		html: string,
		assets: Assets,
		assetTags: AssetTags
	): string {
		const htmlRegExp = /(<html[^>]*>)/i;
		const headRegExp = /(<\/head\s*>)/i;
		const bodyRegExp = /(<\/body\s*>)/i;
		const body = assetTags.bodyTags.map(assetTagObject =>
			htmlTagObjectToString(assetTagObject, options.xhtml)
		);
		const head = assetTags.headTags.map(assetTagObject =>
			htmlTagObjectToString(assetTagObject, options.xhtml)
		);

		if (body.length) {
			if (bodyRegExp.test(html)) {
				// Append assets to body element
				html = html.replace(bodyRegExp, match => body.join("") + match);
			} else {
				// Append scripts to the end of the file if no <body> element exists:
				html += body.join("");
			}
		}

		if (head.length) {
			// Create a head tag if none exists
			if (!headRegExp.test(html)) {
				if (!htmlRegExp.test(html)) {
					html = "<head></head>" + html;
				} else {
					html = html.replace(htmlRegExp, match => match + "<head></head>");
				}
			}

			// Append assets to head element
			html = html.replace(headRegExp, match => head.join("") + match);
		}

		// Inject manifest into the opening html tag
		if (assets.manifest) {
			html = html.replace(/(<html[^>]*)(>)/i, (match, start, end) => {
				// Append the manifest only if no manifest was specified
				if (/\smanifest\s*=/.test(match)) {
					return match;
				}
				return start + ' manifest="' + assets.manifest + '"' + end;
			});
		}
		return html;
	}

	/**
	 * Minify the given string using html-minifier-terser
	 */
	function minifyHtml(html: string) {
		if (typeof options.minify !== "object") {
			return html;
		}
		try {
			return require("html-minifier-terser").minify(html, options.minify);
		} catch (e) {
			if (!(e instanceof Error)) {
				throw e;
			}
			const isParseError = String(e.message).indexOf("Parse Error") === 0;
			if (isParseError) {
				e.message = "html-minifier-terser error:\n" + e.message;
			}
			throw e;
		}
	}
}
