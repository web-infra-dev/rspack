import type { JsAssetInfo } from "@rspack/binding";
import type { PathData } from "../Compilation";

export type FilenameTemplate = string;

export type Filename =
	| FilenameTemplate
	| ((pathData: PathData, assetInfo?: JsAssetInfo) => string);

//#region Name
/** Name of the configuration. Used when loading multiple configurations. */
export type Name = string;
//#endregion

//#region Dependencies
/** A list of name defining all sibling configurations it depends on. Dependent configurations need to be compiled first. */
export type Dependencies = Name[];
//#endregion

//#region Context
/**
 * The context configuration is used to set the base directory for Rspack builds.
 * @default process.cwd()
 * */
export type Context = string;
//#endregion

//#region Mode
/**
 * The mode configuration is used to set the build mode of Rspack to enable the default optimization strategy.
 * @default 'production'
 * */
export type Mode = "development" | "production" | "none";
//#endregion

//#region Falsy
export type Falsy = false | "" | 0 | null | undefined;
//#endregion

//#region Entry
/** The publicPath of the resource referenced by this entry. */
export type PublicPath = "auto" | Filename;

/** The baseURI of the resource referenced by this entry. */
export type BaseUri = string;

/** How this entry load other chunks. */
export type ChunkLoadingType =
	| string
	| "jsonp"
	| "import-scripts"
	| "require"
	| "async-node"
	| "import";

/** How this entry load other chunks. */
export type ChunkLoading = false | ChunkLoadingType;

/** Whether to create a load-on-demand asynchronous chunk for entry. */
export type AsyncChunks = boolean;

/** Option to set the method of loading WebAssembly Modules. */
export type WasmLoadingType =
	| string
	| "fetch-streaming"
	| "fetch"
	| "async-node";

/** Option to set the method of loading WebAssembly Modules. */
export type WasmLoading = false | WasmLoadingType;

export type ScriptType = false | "text/javascript" | "module";

export type LibraryCustomUmdObject = {
	amd?: string;
	commonjs?: string;
	root?: string | string[];
};

/** Specify a name for the library. */
export type LibraryName = string | string[] | LibraryCustomUmdObject;

export type LibraryCustomUmdCommentObject = {
	amd?: string;
	commonjs?: string;
	commonjs2?: string;
	root?: string;
};

/** Use a container(defined in global space) for calling define/require functions in an AMD module. */
export type AmdContainer = string;

/** Add a comment in the UMD wrapper. */
export type AuxiliaryComment = string | LibraryCustomUmdCommentObject;

/** Specify which export should be exposed as a library. */
export type LibraryExport = string | string[];

/** Configure how the library will be exposed. */
export type LibraryType =
	| string
	| "var"
	| "module"
	| "assign"
	| "assign-properties"
	| "this"
	| "window"
	| "self"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "commonjs-static"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system";

/** When using output.library.type: "umd", setting output.library.umdNamedDefine to true will name the AMD module of the UMD build. */
export type UmdNamedDefine = boolean;

/** Options for library.  */
export type LibraryOptions = {
	/** Use a container(defined in global space) for calling define/require functions in an AMD module. */
	amdContainer?: AmdContainer;

	/** Add a comment in the UMD wrapper. */
	auxiliaryComment?: AuxiliaryComment;

	/** Specify which export should be exposed as a library. */
	export?: LibraryExport;

	/** Specify a name for the library. */
	name?: LibraryName;

	/** Configure how the library will be exposed. */
	type: LibraryType;

	/**
	 * When using output.library.type: "umd", setting output.library.umdNamedDefine to true will name the AMD module of the UMD build.
	 * Otherwise, an anonymous define is used.
	 * */
	umdNamedDefine?: UmdNamedDefine;
};

/** Options for library. */
export type Library = LibraryName | LibraryOptions | undefined;

/** The layer of this entry. */
export type Layer = string | null;

/** The filename of the entry chunk. */
export type EntryFilename = Filename;

/** The name of the runtime chunk. */
export type EntryRuntime = false | string;

/** The path to the entry module. */
export type EntryItem = string | string[];

/** The entry that the current entry depends on. With dependOn option you can share the modules from one entry chunk to another. */
export type EntryDependOn = string | string[];

export type EntryDescription = {
	/**
	 * The path to the entry module.
	 * @default './src/index.js'
	 * */
	import: EntryItem;

	/**
	 * The name of the runtime chunk.
	 * When runtime is set, a new runtime chunk will be created.
	 * You can also set it to false to avoid a new runtime chunk.
	 * */
	runtime?: EntryRuntime;

	/** The publicPath of the resource referenced by this entry. */
	publicPath?: PublicPath;

	/** The baseURI of the resource referenced by this entry. */
	baseUri?: BaseUri;

	/** How this entry load other chunks. */
	chunkLoading?: ChunkLoading;

	/** Whether to create a load-on-demand asynchronous chunk for this entry. */
	asyncChunks?: AsyncChunks;

	/** Option to set the method of loading WebAssembly Modules. */
	wasmLoading?: WasmLoading;

	/** The filename of the entry chunk. */
	filename?: EntryFilename;

	/** The format of the chunk generated by this entry as a library. */
	library?: LibraryOptions;

	/** The entry that the current entry depends on. With dependOn option you can share the modules from one entry chunk to another. */
	dependOn?: EntryDependOn;

	/** The layer of this entry, make the corresponding configuration take effect through layer matching in SplitChunks, Rules, Stats, and Externals. */
	layer?: Layer;
};

export type EntryUnnamed = EntryItem;

export type EntryObject = Record<string, EntryItem | EntryDescription>;

/** A static entry.  */
export type EntryStatic = EntryObject | EntryUnnamed;

/** A Function returning entry options. */
export type EntryDynamic = () => EntryStatic | Promise<EntryStatic>;

/** The entry options for building */
export type Entry = EntryStatic | EntryDynamic;
//#endregion

//#region Resolve
/**
 * Path alias
 * @example
 * ```js
 * {
 * 	"@": path.resolve(__dirname, './src'),
 * 	"abc$": path.resolve(__dirname, './node_modules/abc/index.js'),
 * }
 * // - require("@/a") will attempt to resolve <root>/src/a.
 * // - require("abc") will attempt to resolve <root>/src/abc.
 * // - require("abc/file.js") will not match, and it will attempt to resolve node_modules/abc/file.js.
 * ```
 * */
export type ResolveAlias = {
	[x: string]: string | false | (string | false)[];
};

/** The replacement of [tsconfig-paths-webpack-plugin](https://www.npmjs.com/package/tsconfig-paths-webpack-plugin) in Rspack. */
export type ResolveTsConfig =
	| string
	| {
			configFile: string;
			references?: string[] | "auto" | undefined;
	  };

/** Used to configure the Rspack module resolution */
export type ResolveOptions = {
	/** Path alias */
	alias?: ResolveAlias;

	/** Same as node's [conditionNames](https://nodejs.org/api/packages.html#conditional-exports) for the exports and imports fields in package.json. */
	conditionNames?: string[];

	/**
	 * Parse modules in order.
	 * @default [".js", ".json", ".wasm"]
	 * */
	extensions?: string[];

	/** Redirect module requests when normal resolving fails. */
	fallback?: ResolveAlias;

	/** Try to parse the fields in package.json */
	mainFields?: string[];

	/**
	 * The filename suffix when resolving directories, e.g. require('. /dir/') will try to resolve '. /dir/index'.
	 * @default ['index']
	 */
	mainFiles?: string[];

	/**
	 * The name of the directory to use when resolving dependencies.
	 * @default ["node_modules"]
	 */
	modules?: string[];

	/**
	 * When enabled, require('file') will first look for the . /file file in the current directory, not <modules>/file.
	 * @default false
	 */
	preferRelative?: boolean;

	/**
	 * Opt for absolute paths when resolving, in relation to resolve.roots.
	 * @default false
	 */
	preferAbsolute?: boolean;

	/**
	 * Whether to resolve symlinks to their symlinked location.
	 * @default true
	 */
	symlinks?: boolean;

	/**
	 * By default, It changes to true if resolve.extensions contains an empty string;
	 * otherwise, this value changes to false.
	 */
	enforceExtension?: boolean;

	/**
	 * Customize the imports field in package.json which are used to provide the internal requests of a package (requests starting with # are considered internal).
	 * @default ["imports"]
	 */
	importsFields?: string[];

	/**
	 * The JSON files to use for descriptions.
	 * @default ['package.json']
	 */
	descriptionFiles?: string[];

	/** The replacement of [tsconfig-paths-webpack-plugin](https://www.npmjs.com/package/tsconfig-paths-webpack-plugin) in Rspack. */
	tsConfig?: ResolveTsConfig;

	/**
	 * No longer resolve extensions, no longer resolve mainFiles in package.json (but does not affect requests from mainFiles, browser, alias).
	 * @default false
	 * */
	fullySpecified?: boolean;

	/**
	 * Customize the exports field in package.json.
	 * @default ["exports"]
	 * */
	exportsFields?: string[];

	/** Define alias for the extension. */
	extensionAlias?: Record<string, string | string[]>;

	/**
	 * Define a field, such as browser, that should be parsed in accordance with this [specification](https://github.com/defunctzombie/package-browser-field-spec).
	 * @default ['browser']
	 * */
	aliasFields?: string[];

	/**
	 * A list of resolve restrictions to restrict the paths that a request can be resolved on.
	 * @default []
	 * */
	restrictions?: string[];

	/**
	 * A list of directories where server-relative URLs (beginning with '/') are resolved.
	 * It defaults to the context configuration option.
	 * On systems other than Windows, these requests are initially resolved as an absolute path.
	 * @default []
	 */
	roots?: string[];

	/** Customize the Resolve configuration based on the module type. */
	byDependency?: Record<string, ResolveOptions>;
};

/** Used to configure the Rspack module resolution */
export type Resolve = ResolveOptions;
//#endregion

//#region ExternalsType
/**
 * Specify the default type of externals.
 * `amd`, `umd`, `system` and `jsonp` externals depend on the `output.libraryTarget` being set to the same value e.g. you can only consume amd externals within an amd library.
 * @default 'var'
 */
export type ExternalsType =
	| "var"
	| "module"
	| "assign"
	| "this"
	| "window"
	| "self"
	| "global"
	| "commonjs"
	| "commonjs2"
	| "commonjs-module"
	| "commonjs-static"
	| "amd"
	| "amd-require"
	| "umd"
	| "umd2"
	| "jsonp"
	| "system"
	| "promise"
	| "import"
	| "module-import"
	| "script"
	| "node-commonjs";
//#endregion

//#region Externals
/**
 * The dependency used for the external.
 */
export type ExternalItemValue =
	| string
	| boolean
	| string[]
	| Record<string, string | string[]>;

/**
 * If an dependency matches exactly a property of the object, the property value is used as dependency.
 */
export type ExternalItemObjectUnknown = {
	[x: string]: ExternalItemValue;
};

/**
 * Data object passed as argument when a function is set for 'externals'.
 */
export type ExternalItemFunctionData = {
	context?: string;
	dependencyType?: string;
	request?: string;
	contextInfo?: {
		issuer: string;
	};
};

/**
 * Prevent bundling of certain imported package and instead retrieve these external dependencies at runtime.
 *
 * @example
 * ```js
 * // jquery lib will be excluded from bundling.
 * module.exports = {
 * 	externals: {
 * 		jquery: 'jQuery',
 * 	}
 * }
 * ```
 * */
export type ExternalItem =
	| string
	| RegExp
	| ExternalItemObjectUnknown
	| ((
			data: ExternalItemFunctionData,
			callback: (
				err?: Error,
				result?: ExternalItemValue,
				type?: ExternalsType
			) => void
	  ) => void)
	| ((data: ExternalItemFunctionData) => Promise<ExternalItemValue>);

/**
 * Prevent bundling of certain imported packages and instead retrieve these external dependencies at runtime.
 *
 * @example
 * ```js
 * // jquery lib will be excluded from bundling.
 * module.exports = {
 * 	externals: {
 * 		jquery: 'jQuery',
 * 	}
 * }
 * ```
 * */
export type Externals = ExternalItem | ExternalItem[];
//#endregion
