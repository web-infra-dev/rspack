module.exports = {
	definitions: {
		AssetModuleFilename: {
			description:
				"The filename of asset modules as relative path inside the 'output.path' directory.",
			anyOf: [
				{
					type: "string"
				}
			]
		},
		AssetParserDataUrlOptions: {
			description: "Options object for DataUrl condition.",
			type: "object",
			additionalProperties: false,
			properties: {
				maxSize: {
					description:
						"Maximum size of asset that should be inline as modules. Default: 8kb.",
					type: "number"
				}
			}
		},
		AssetParserOptions: {
			description: "Parser options for asset modules.",
			type: "object",
			additionalProperties: false,
			properties: {
				dataUrlCondition: {
					description: "The condition for inlining the asset as DataUrl.",
					anyOf: [
						{
							$ref: "#/definitions/AssetParserDataUrlOptions"
						}
					]
				}
			}
		},
		AuxiliaryComment: {
			description: "Add a comment in the UMD wrapper.",
			anyOf: [
				{
					description: "Append the same comment above each import style.",
					type: "string"
				},
				{
					$ref: "#/definitions/LibraryCustomUmdCommentObject"
				}
			]
		},
		CacheOptions: {
			description:
				"Cache generated modules and chunks to improve performance for multiple incremental builds.",
			type: "boolean"
		},
		ChunkFilename: {
			description:
				"Specifies the filename template of output files of non-initial chunks on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		ChunkFormat: {
			description:
				"The format of chunks (formats included by default are 'array-push' (web/WebWorker), 'commonjs' (node.js), 'module' (ESM), but others might be added by plugins).",
			anyOf: [
				{
					enum: ["array-push", "commonjs", "module", false]
				},
				{
					type: "string"
				}
			]
		},
		ChunkLoading: {
			description:
				"The method of loading chunks (methods included by default are 'jsonp' (web), 'import' (ESM), 'importScripts' (WebWorker), 'require' (sync node.js), 'async-node' (async node.js), but others might be added by plugins).",
			anyOf: [
				{
					enum: [false]
				},
				{
					$ref: "#/definitions/ChunkLoadingType"
				}
			]
		},
		ChunkLoadingType: {
			description:
				"The method of loading chunks (methods included by default are 'jsonp' (web), 'import' (ESM), 'importScripts' (WebWorker), 'require' (sync node.js), 'async-node' (async node.js), but others might be added by plugins).",
			anyOf: [
				{
					enum: ["jsonp", "import-scripts", "require", "async-node", "import"]
				},
				{
					type: "string"
				}
			]
		},
		CrossOriginLoading: {
			description: "This option enables cross-origin loading of chunks.",
			enum: [false, "anonymous", "use-credentials"]
		},
		Context: {
			description:
				"The base directory (absolute path!) for resolving the `entry` option. If `output.pathinfo` is set, the included pathinfo is shortened to this directory.",
			type: "string"
		},
		CssChunkFilename: {
			description:
				"Specifies the filename template of non-initial output css files on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		CssFilename: {
			description:
				"Specifies the filename template of output css files on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		HotUpdateChunkFilename: {
			description:
				"The filename of the Hot Update Chunks. They are inside the output.path directory.",
			type: "string",
			absolutePath: false
		},
		HotUpdateMainFilename: {
			description:
				"The filename of the Hot Update Main File. It is inside the 'output.path' directory.",
			type: "string",
			absolutePath: false
		},
		WebassemblyModuleFilename: {
			description:
				"The filename of WebAssembly modules as relative path inside the 'output.path' directory.",
			type: "string"
		},
		EnabledWasmLoadingTypes: {
			description:
				"List of wasm loading types enabled for use by entry points.",
			type: "array",
			items: {
				$ref: "#/definitions/WasmLoadingType"
			}
		},
		EnabledChunkLoadingTypes: {
			description:
				"List of chunk loading types enabled for use by entry points.",
			type: "array",
			items: {
				$ref: "#/definitions/ChunkLoadingType"
			}
		},
		WasmLoading: {
			description:
				"The method of loading WebAssembly Modules (methods included by default are 'fetch' (web/WebWorker), 'async-node' (node.js), but others might be added by plugins).",
			anyOf: [
				{
					enum: [false]
				},
				{
					$ref: "#/definitions/WasmLoadingType"
				}
			]
		},
		WasmLoadingType: {
			description:
				"The method of loading WebAssembly Modules (methods included by default are 'fetch' (web/WebWorker), 'async-node' (node.js), but others might be added by plugins).",
			anyOf: [
				{
					enum: ["fetch-streaming", "fetch", "async-node"]
				},
				{
					type: "string"
				}
			]
		},
		Dependencies: {
			description: "References to other configurations to depend on.",
			type: "array",
			items: {
				description: "References to another configuration to depend on.",
				type: "string"
			}
		},
		DevServer: {
			description: "Options for the rspack-dev-server.",
			type: "object"
		},
		DevTool: {
			description:
				"A developer tool to enhance debugging (false | eval | [inline-|hidden-|eval-][nosources-][cheap-[module-]]source-map).",
			anyOf: [
				{
					enum: [false]
				},
				{
					type: "string",
					pattern:
						"^(inline-|hidden-|eval-)?(nosources-)?(cheap-(module-)?)?source-map$"
				}
			]
		},
		EnabledLibraryTypes: {
			description: "List of library types enabled for use by entry points.",
			type: "array",
			items: {
				$ref: "#/definitions/LibraryType"
			}
		},
		Entry: {
			description: "The entry point(s) of the compilation.",
			anyOf: [
				{
					$ref: "#/definitions/EntryStatic"
				}
			]
		},
		EntryDescription: {
			description: "An object with entry point description.",
			type: "object",
			additionalProperties: false,
			properties: {
				import: {
					$ref: "#/definitions/EntryItem"
				},
				runtime: {
					$ref: "#/definitions/EntryRuntime"
				},
				wasmLoading: {
					$ref: "#/definitions/WasmLoading"
				}
			},
			required: ["import"]
		},
		EntryFilename: {
			description:
				"Specifies the filename of the output file on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		EntryItem: {
			description: "Module(s) that are loaded upon startup.",
			anyOf: [
				{
					description:
						"All modules are loaded upon startup. The last one is exported.",
					type: "array",
					items: {
						description:
							"A module that is loaded upon startup. Only the last one is exported.",
						type: "string",
						minLength: 1
					},
					minItems: 1,
					uniqueItems: true
				},
				{
					description:
						"The string is resolved to a module which is loaded upon startup.",
					type: "string",
					minLength: 1
				}
			]
		},
		EntryObject: {
			description:
				"Multiple entry bundles are created. The key is the entry name. The value can be a string, an array or an entry description object.",
			type: "object",
			additionalProperties: {
				description: "An entry point with name.",
				anyOf: [
					{
						$ref: "#/definitions/EntryItem"
					},
					{
						$ref: "#/definitions/EntryDescription"
					}
				]
			}
		},
		EntryRuntime: {
			description:
				"The name of the runtime chunk. If set a runtime chunk with this name is created or an existing entrypoint is used as runtime.",
			anyOf: [
				{
					enum: [false]
				},
				{
					type: "string",
					minLength: 1
				}
			]
		},
		EntryStatic: {
			description: "A static entry description.",
			anyOf: [
				{
					$ref: "#/definitions/EntryObject"
				},
				{
					$ref: "#/definitions/EntryUnnamed"
				}
			]
		},
		EntryUnnamed: {
			description: "An entry point without name.",
			oneOf: [
				{
					$ref: "#/definitions/EntryItem"
				}
			]
		},
		Experiments: {
			description:
				"Enables/Disables experiments (experimental features with relax SemVer compatibility).",
			type: "object",
			additionalProperties: false,
			properties: {
				asyncWebAssembly: {
					description: "Support WebAssembly as asynchronous EcmaScript Module.",
					type: "boolean"
				},
				incrementalRebuild: {
					description: "Rebuild incrementally",
					type: "boolean"
				},
				lazyCompilation: {
					description:
						"Compile entrypoints and import()s only when they are accessed.",
					anyOf: [
						{
							type: "boolean"
						}
					]
				},
				outputModule: {
					description: "Allow output javascript files as module source type.",
					type: "boolean"
				},
				newSplitChunks: {
					description: "Enable new SplitChunksPlugin",
					type: "boolean"
				},
				css: {
					description: "Enable native css support.",
					type: "boolean"
				}
			}
		},
		ExternalItem: {
			description:
				"Specify dependency that shouldn't be resolved by rspack, but should become dependencies of the resulting bundle. The kind of the dependency depends on `output.libraryTarget`.",
			anyOf: [
				{
					description: "Every matched dependency becomes external.",
					instanceof: "RegExp"
				},
				{
					description:
						"An exact matched dependency becomes external. The same string is used as external dependency.",
					type: "string"
				},
				{
					description:
						"If an dependency matches exactly a property of the object, the property value is used as dependency.",
					type: "object",
					additionalProperties: {
						$ref: "#/definitions/ExternalItemValue"
					}
				},
				{
					description:
						"The function is called on each dependency (`function(context, request, callback(err, result))`).",
					instanceof: "Function"
					// tsType:
					// 	"(((data: ExternalItemFunctionData, callback: (err?: Error, result?: ExternalItemValue) => void) => void) | ((data: ExternalItemFunctionData) => Promise<ExternalItemValue>))"
				}
			]
		},
		ExternalItemValue: {
			description: "The dependency used for the external.",
			anyOf: [
				{
					type: "array",
					items: {
						description: "A part of the target of the external.",
						type: "string",
						minLength: 1
					}
				},
				{
					description: "The target of the external.",
					type: "string"
				},
				{
					description:
						"`true`: The dependency name is used as target of the external.",
					type: "boolean"
				}
			]
		},
		Externals: {
			description:
				"Specify dependencies that shouldn't be resolved by rspack, but should become dependencies of the resulting bundle. The kind of the dependency depends on `output.libraryTarget`.",
			anyOf: [
				{
					type: "array",
					items: {
						$ref: "#/definitions/ExternalItem"
					}
				},
				{
					$ref: "#/definitions/ExternalItem"
				}
			]
		},
		ExternalsPresets: {
			description: "Enable presets of externals for specific targets.",
			type: "object",
			additionalProperties: false,
			properties: {
				node: {
					description:
						"Treat node.js built-in modules like fs, path or vm as external and load them via require() when used.",
					type: "boolean"
				}
			}
		},
		ExternalsType: {
			description:
				"Specifies the default type of externals ('amd*', 'umd*', 'system' and 'jsonp' depend on output.libraryTarget set to the same value).",
			enum: [
				"var",
				"module",
				"assign",
				"this",
				"window",
				"self",
				"global",
				"commonjs",
				"commonjs2",
				"commonjs-module",
				"commonjs-static",
				"amd",
				"amd-require",
				"umd",
				"umd2",
				"jsonp",
				"system",
				"promise",
				"import",
				"script",
				"node-commonjs"
			]
		},
		Filename: {
			description:
				"Specifies the filename of output files on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		SourceMapFilename: {
			description:
				"Specifies the filename of output files on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			oneOf: [
				{
					$ref: "#/definitions/FilenameTemplate"
				}
			]
		},
		FilenameTemplate: {
			description:
				"Specifies the filename template of output files on disk. You must **not** specify an absolute path here, but the path may contain folders separated by '/'! The specified path is joined with the value of the 'output.path' option to determine the location on disk.",
			anyOf: [
				{
					type: "string",
					minLength: 1
				},
				{
					instanceof: "Function"
				}
			]
		},
		FilterItemTypes: {
			description: "Filtering value, regexp or function.",
			anyOf: [
				{
					instanceof: "RegExp"
				},
				{
					type: "string"
				},
				{
					instanceof: "Function"
				}
			]
		},
		FilterTypes: {
			description: "Filtering values.",
			anyOf: [
				{
					type: "array",
					items: {
						description: "Rule to filter.",
						oneOf: [
							{
								$ref: "#/definitions/FilterItemTypes"
							}
						]
					}
				},
				{
					$ref: "#/definitions/FilterItemTypes"
				}
			]
		},
		GlobalObject: {
			description:
				"An expression which is used to address the global object/scope in runtime code.",
			type: "string",
			minLength: 1
		},
		ImportFunctionName: {
			description:
				"The name of the native import() function (can be exchanged for a polyfill).",
			type: "string"
		},
		InfrastructureLogging: {
			description: "Options for infrastructure level logging.",
			type: "object",
			additionalProperties: false,
			properties: {
				appendOnly: {
					description:
						"Only appends lines to the output. Avoids updating existing output e. g. for status messages. This option is only used when no custom console is provided.",
					type: "boolean"
				},
				colors: {
					description:
						"Enables/Disables colorful output. This option is only used when no custom console is provided.",
					type: "boolean"
				},
				console: {
					description: "Custom console used for logging."
				},
				debug: {
					description: "Enable debug logging for specific loggers.",
					anyOf: [
						{
							description: "Enable/Disable debug logging for all loggers.",
							type: "boolean"
						},
						{
							$ref: "#/definitions/FilterTypes"
						}
					]
				},
				level: {
					description: "Log level.",
					enum: ["none", "error", "warn", "info", "log", "verbose"]
				},
				stream: {
					description:
						"Stream used for logging output. Defaults to process.stderr. This option is only used when no custom console is provided."
				}
			}
		},
		Library: {
			description:
				"Make the output files a library, exporting the exports of the entry point.",
			anyOf: [
				{
					$ref: "#/definitions/LibraryName"
				},
				{
					$ref: "#/definitions/LibraryOptions"
				}
			]
		},
		LibraryCustomUmdCommentObject: {
			description:
				"Set explicit comments for `commonjs`, `commonjs2`, `amd`, and `root`.",
			type: "object",
			additionalProperties: false,
			properties: {
				amd: {
					description: "Set comment for `amd` section in UMD.",
					type: "string"
				},
				commonjs: {
					description: "Set comment for `commonjs` (exports) section in UMD.",
					type: "string"
				},
				commonjs2: {
					description:
						"Set comment for `commonjs2` (module.exports) section in UMD.",
					type: "string"
				},
				root: {
					description:
						"Set comment for `root` (global variable) section in UMD.",
					type: "string"
				}
			}
		},
		LibraryCustomUmdObject: {
			description:
				"Description object for all UMD variants of the library name.",
			type: "object",
			additionalProperties: false,
			properties: {
				amd: {
					description: "Name of the exposed AMD library in the UMD.",
					type: "string",
					minLength: 1
				},
				commonjs: {
					description: "Name of the exposed commonjs export in the UMD.",
					type: "string",
					minLength: 1
				},
				root: {
					description:
						"Name of the property exposed globally by a UMD library.",
					anyOf: [
						{
							type: "array",
							items: {
								description:
									"Part of the name of the property exposed globally by a UMD library.",
								type: "string",
								minLength: 1
							}
						},
						{
							type: "string",
							minLength: 1
						}
					]
				}
			}
		},
		LibraryExport: {
			description: "Specify which export should be exposed as library.",
			anyOf: [
				{
					type: "array",
					items: {
						description:
							"Part of the export that should be exposed as library.",
						type: "string",
						minLength: 1
					}
				},
				{
					type: "string",
					minLength: 1
				}
			]
		},
		LibraryName: {
			description:
				"The name of the library (some types allow unnamed libraries too).",
			anyOf: [
				{
					type: "array",
					items: {
						description: "A part of the library name.",
						type: "string",
						minLength: 1
					},
					minItems: 1
				},
				{
					type: "string",
					minLength: 1
				},
				{
					$ref: "#/definitions/LibraryCustomUmdObject"
				}
			]
		},
		LibraryOptions: {
			description: "Options for library.",
			type: "object",
			additionalProperties: false,
			properties: {
				auxiliaryComment: {
					$ref: "#/definitions/AuxiliaryComment"
				},
				export: {
					$ref: "#/definitions/LibraryExport"
				},
				name: {
					$ref: "#/definitions/LibraryName"
				},
				type: {
					$ref: "#/definitions/LibraryType"
				},
				umdNamedDefine: {
					$ref: "#/definitions/UmdNamedDefine"
				}
			},
			required: ["type"]
		},
		LibraryType: {
			description:
				"Type of library (types included by default are 'var', 'module', 'assign', 'assign-properties', 'this', 'window', 'self', 'global', 'commonjs', 'commonjs2', 'commonjs-module', 'commonjs-static', 'amd', 'amd-require', 'umd', 'umd2', 'jsonp', 'system', but others might be added by plugins).",
			anyOf: [
				{
					enum: [
						"var",
						"module",
						"assign",
						"assign-properties",
						"this",
						"window",
						"self",
						"global",
						"commonjs",
						"commonjs2",
						"commonjs-module",
						"commonjs-static",
						"amd",
						"amd-require",
						"umd",
						"umd2",
						"jsonp",
						"system"
					]
				},
				{
					type: "string"
				}
			]
		},
		Mode: {
			description: "Enable production optimizations or development hints.",
			enum: ["development", "production", "none"]
		},
		ignoreWarnings: {
			description: "ignore warnings based on pattern",
			type: "array",
			items: {
				anyOf: [
					{
						instanceof: "RegExp"
					},
					{
						instanceof: "Function"
						// tsType:
						// 	"(warning: Error, compilation: Compilation) => boolean"
					}
				]
			}
		},
		ModuleOptions: {
			description:
				"Options affecting the normal modules (`NormalModuleFactory`).",
			type: "object",
			additionalProperties: false,
			properties: {
				defaultRules: {
					description: "An array of rules applied by default for modules.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetRules"
						}
					]
				},
				parser: {
					$ref: "#/definitions/ParserOptionsByModuleType"
				},
				rules: {
					description: "An array of rules applied for modules.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetRules"
						}
					]
				}
			}
		},
		Name: {
			description:
				"Name of the configuration. Used when loading multiple configurations.",
			type: "string"
		},
		Node: {
			description: "Include polyfills or mocks for various node stuff.",
			anyOf: [
				{
					enum: [false]
				},
				{
					$ref: "#/definitions/NodeOptions"
				}
			]
		},
		NodeOptions: {
			description: "Options object for node compatibility features.",
			type: "object",
			additionalProperties: false,
			properties: {
				__dirname: {
					description: "Include a polyfill for the '__dirname' variable.",
					enum: [false, true, "warn-mock", "mock", "eval-only"]
				},
				__filename: {
					description: "Include a polyfill for the '__filename' variable.",
					enum: [false, true, "warn-mock", "mock", "eval-only"]
				},
				global: {
					description: "Include a polyfill for the 'global' variable.",
					enum: [false, true, "warn"]
				}
			}
		},
		Optimization: {
			description: "Enables/Disables integrated optimizations.",
			type: "object",
			additionalProperties: false,
			properties: {
				chunkIds: {
					description:
						"Define the algorithm to choose chunk ids (named: readable ids for better debugging, deterministic: numeric hash ids for better long term caching, size: numeric ids focused on minimal initial download size, total-size: numeric ids focused on minimal total download size, false: no algorithm used, as custom one can be provided via plugin).",
					enum: ["named", "deterministic"]
				},
				minimize: {
					description:
						"Enable minimizing the output. Uses optimization.minimizer.",
					type: "boolean"
				},
				minimizer: {
					description: "Minimizer(s) to use for minimizing the output.",
					type: "array",
					items: {
						description: "Plugin of type object or instanceof Function.",
						anyOf: [
							{
								enum: ["..."]
							},
							{
								$ref: "#/definitions/RspackPluginInstance"
							},
							{
								$ref: "#/definitions/RspackPluginFunction"
							}
						]
					}
				},
				moduleIds: {
					description:
						"Define the algorithm to choose module ids (natural: numeric ids in order of usage, named: readable ids for better debugging, hashed: (deprecated) short hashes as ids for better long term caching, deterministic: numeric hash ids for better long term caching, size: numeric ids focused on minimal initial download size, false: no algorithm used, as custom one can be provided via plugin).",
					enum: ["named", "deterministic"]
				},
				removeAvailableModules: {
					description:
						"Removes modules from chunks when these modules are already included in all parents.",
					type: "boolean"
				},
				removeEmptyChunks: {
					description: "Remove chunks which are empty.",
					type: "boolean"
				},
				runtimeChunk: {
					$ref: "#/definitions/OptimizationRuntimeChunk"
				},
				sideEffects: {
					description:
						"Skip over modules which contain no side effects when exports are not used (false: disabled, 'flag': only use manually placed side effects flag, true: also analyse source code for side effects).",
					anyOf: [
						{
							enum: ["flag"]
						},
						{
							type: "boolean"
						}
					]
				},
				splitChunks: {
					description:
						"Optimize duplication and caching by splitting chunks by shared modules and cache group.",
					anyOf: [
						{
							enum: [false]
						},
						{
							$ref: "#/definitions/OptimizationSplitChunksOptions"
						}
					]
				},
				realContentHash: {
					description:
						"Use real [contenthash] based on final content of the assets.",
					type: "boolean"
				}
			}
		},
		OptimizationRuntimeChunk: {
			description:
				"Create an additional chunk which contains only the rspack runtime and chunk hash maps.",
			anyOf: [
				{
					enum: ["single", "multiple"]
				},
				{
					type: "boolean"
				},
				{
					type: "object",
					additionalProperties: false,
					properties: {
						name: {
							description: "The name or name factory for the runtime chunks.",
							anyOf: [
								{
									type: "string"
								},
								{
									instanceof: "Function"
								}
							]
						}
					}
				}
			]
		},
		OptimizationSplitChunksCacheGroup: {
			description:
				"Options object for describing behavior of a cache group selecting modules that should be cached together.",
			type: "object",
			additionalProperties: false,
			properties: {
				chunks: {
					description:
						'Select chunks for determining cache group content (defaults to "initial", "initial" and "all" requires adding these chunks to the HTML).',
					anyOf: [
						{
							enum: ["initial", "async", "all"]
						},
						{
							instanceof: "Function"
						}
					]
				},
				minChunks: {
					description:
						"Minimum number of times a module has to be duplicated until it's considered for splitting.",
					type: "number",
					minimum: 1
				},
				name: {
					description:
						"Give chunks for this cache group a name (chunks with equal name are merged).",
					anyOf: [
						{
							enum: [false]
						},
						{
							type: "string"
						},
						{
							instanceof: "Function"
						}
					]
				},
				priority: {
					description: "Priority of this cache group.",
					type: "number"
				},
				reuseExistingChunk: {
					description:
						"Try to reuse existing chunk (with name) when it has matching modules.",
					type: "boolean"
				},
				enforce: {
					description:
						"ignore splitChunks.minSize, splitChunks.minChunks, splitChunks.maxAsyncRequests and splitChunks.maxInitialRequests options and always create chunks for this cache group.",
					type: "boolean"
				},
				hidePathInfo: {
					type: "boolean"
				},
				maxSize: {
					type: "number"
				},
				test: {
					description: "Assign modules to a cache group by module name.",
					anyOf: [
						{
							instanceof: "RegExp"
						}
					]
				},
				minSize: {
					description: "Minimal size for the created chunks.",
					oneOf: [
						{
							$ref: "#/definitions/OptimizationSplitChunksSizes"
						}
					]
				}
			}
		},
		OptimizationSplitChunksOptions: {
			description: "Options object for splitting chunks into smaller chunks.",
			type: "object",
			additionalProperties: false,
			properties: {
				fallbackCacheGroup: {
					type: "object",
					properties: {
						maxSize: {
							type: "number"
						},
						maxInitialSize: {
							type: "number"
						},
						maxAsyncSize: {
							type: "number"
						},
						minSize: {
							type: "number"
						}
					}
				},
				hidePathInfo: {
					type: "boolean"
				},
				name: {
					description: "The name or name for chunks.",
					anyOf: [
						{
							type: "string"
						}
					]
				},
				cacheGroups: {
					description:
						"Assign modules to a cache group (modules from different cache groups are tried to keep in separate chunks, default categories: 'default', 'defaultVendors').",
					type: "object",
					additionalProperties: {
						description: "Configuration for a cache group.",
						anyOf: [
							{
								$ref: "#/definitions/OptimizationSplitChunksCacheGroup"
							}
						]
					}
				},
				chunks: {
					description:
						'Select chunks for determining shared modules (defaults to "async", "initial" and "all" requires adding these chunks to the HTML).',
					anyOf: [
						{
							enum: ["initial", "async", "all"]
						}
					]
				},
				enforceSizeThreshold: {
					description:
						"Size threshold at which splitting is enforced and other restrictions (minRemainingSize, maxAsyncRequests, maxInitialRequests) are ignored.",
					oneOf: [
						{
							$ref: "#/definitions/OptimizationSplitChunksSizes"
						}
					]
				},
				maxAsyncRequests: {
					description:
						"Maximum number of requests which are accepted for on-demand loading.",
					type: "number",
					minimum: 1
				},
				maxInitialRequests: {
					description:
						"Maximum number of initial chunks which are accepted for an entry point.",
					type: "number",
					minimum: 1
				},
				minChunks: {
					description:
						"Minimum number of times a module has to be duplicated until it's considered for splitting.",
					type: "number",
					minimum: 1
				},
				minRemainingSize: {
					description:
						"Minimal size for the chunks the stay after moving the modules to a new chunk.",
					oneOf: [
						{
							$ref: "#/definitions/OptimizationSplitChunksSizes"
						}
					]
				},
				minSize: {
					description: "Minimal size for the created chunks.",
					oneOf: [
						{
							$ref: "#/definitions/OptimizationSplitChunksSizes"
						}
					]
				},
				maxSize: {
					type: "number"
				},
				maxInitialSize: {
					type: "number"
				},
				maxAsyncSize: {
					type: "number"
				},
				reuseExistingChunk: {
					description:
						"If the current chunk contains modules already split out from the main bundle, it will be reused instead of a new one being generated. This can affect the resulting file name of the chunk.",
					type: "boolean"
				}
			}
		},
		OptimizationSplitChunksSizes: {
			description: "Size description for limits.",
			anyOf: [
				{
					description: "Size of the javascript part of the chunk.",
					type: "number",
					minimum: 0
				}
			]
		},
		Iife: {
			description:
				"Wrap javascript code into IIFE's to avoid leaking into global scope.",
			type: "boolean"
		},
		Clean: {
			description: "Clears the output build directory",
			type: "boolean"
		},
		Output: {
			description:
				"Options affecting the output of the compilation. `output` options tell rspack how to write the compiled files to disk.",
			type: "object",
			additionalProperties: false,
			properties: {
				iife: {
					$ref: "#/definitions/Iife"
				},
				clean: {
					$ref: "#/definitions/Clean"
				},
				assetModuleFilename: {
					$ref: "#/definitions/AssetModuleFilename"
				},
				auxiliaryComment: {
					oneOf: [
						{
							$ref: "#/definitions/AuxiliaryComment"
						}
					]
				},
				chunkFormat: {
					$ref: "#/definitions/ChunkFormat"
				},
				chunkLoading: {
					$ref: "#/definitions/ChunkLoading"
				},
				enabledChunkLoadingTypes: {
					$ref: "#/definitions/EnabledChunkLoadingTypes"
				},
				chunkFilename: {
					$ref: "#/definitions/ChunkFilename"
				},
				crossOriginLoading: {
					$ref: "#/definitions/CrossOriginLoading"
				},
				cssChunkFilename: {
					$ref: "#/definitions/CssChunkFilename"
				},
				cssFilename: {
					$ref: "#/definitions/CssFilename"
				},
				hotUpdateChunkFilename: {
					$ref: "#/definitions/HotUpdateChunkFilename"
				},
				hotUpdateMainFilename: {
					$ref: "#/definitions/HotUpdateMainFilename"
				},
				enabledWasmLoadingTypes: {
					$ref: "#/definitions/EnabledWasmLoadingTypes"
				},
				wasmLoading: {
					$ref: "#/definitions/WasmLoading"
				},
				webassemblyModuleFilename: {
					$ref: "#/definitions/WebassemblyModuleFilename"
				},
				enabledLibraryTypes: {
					$ref: "#/definitions/EnabledLibraryTypes"
				},
				filename: {
					$ref: "#/definitions/Filename"
				},
				globalObject: {
					$ref: "#/definitions/GlobalObject"
				},
				importFunctionName: {
					$ref: "#/definitions/ImportFunctionName"
				},
				library: {
					$ref: "#/definitions/Library"
				},
				libraryExport: {
					oneOf: [
						{
							$ref: "#/definitions/LibraryExport"
						}
					]
				},
				libraryTarget: {
					oneOf: [
						{
							$ref: "#/definitions/LibraryType"
						}
					]
				},
				module: {
					$ref: "#/definitions/OutputModule"
				},
				path: {
					$ref: "#/definitions/Path"
				},
				publicPath: {
					$ref: "#/definitions/PublicPath"
				},
				strictModuleErrorHandling: {
					$ref: "#/definitions/StrictModuleErrorHandling"
				},
				umdNamedDefine: {
					oneOf: [
						{
							$ref: "#/definitions/UmdNamedDefine"
						}
					]
				},
				uniqueName: {
					$ref: "#/definitions/UniqueName"
				},
				chunkLoadingGlobal: {
					$ref: "#/definitions/ChunkLoadingGlobal"
				},
				trustedTypes: {
					description:
						"Use a Trusted Types policy to create urls for chunks. 'output.uniqueName' is used a default policy name. Passing a string sets a custom policy name.",
					anyOf: [
						{
							enum: [true]
						},
						{
							description:
								"The name of the Trusted Types policy created by webpack to serve bundle chunks.",
							type: "string",
							minLength: 1
						},
						{
							$ref: "#/definitions/TrustedTypes"
						}
					]
				},
				sourceMapFilename: {
					$ref: "#/definitions/SourceMapFilename"
				}
			}
		},
		OutputModule: {
			description: "Output javascript files as module source type.",
			type: "boolean"
		},
		ParserOptionsByModuleType: {
			description: "Specify options for each parser.",
			type: "object",
			additionalProperties: {
				description: "Options for parsing.",
				type: "object",
				additionalProperties: true
			},
			properties: {
				asset: {
					$ref: "#/definitions/AssetParserOptions"
				}
			}
		},
		Path: {
			description: "The output directory as **absolute path** (required).",
			type: "string"
		},
		Plugins: {
			description: "Add additional plugins to the compiler.",
			type: "array",
			items: {
				description: "Plugin of type object or instanceof Function.",
				anyOf: [
					{
						$ref: "#/definitions/RspackPluginInstance"
					},
					{
						$ref: "#/definitions/RspackPluginFunction"
					}
				]
			}
		},
		PublicPath: {
			description:
				"The 'publicPath' specifies the public URL address of the output files when referenced in a browser.",
			anyOf: [
				{
					enum: ["auto"]
				},
				{
					$ref: "#/definitions/RawPublicPath"
				}
			]
		},
		RawPublicPath: {
			description:
				"The 'publicPath' specifies the public URL address of the output files when referenced in a browser.",
			anyOf: [
				{
					type: "string"
				}
			]
		},
		Resolve: {
			description: "Options for the resolver.",
			oneOf: [
				{
					$ref: "#/definitions/ResolveOptions"
				}
			]
		},
		ResolveAlias: {
			description: "Redirect module requests.",
			anyOf: [
				{
					type: "object",
					additionalProperties: {
						description: "New request.",
						anyOf: [
							{
								description: "Multiple alternative requests.",
								type: "array",
								items: {
									description: "One choice of request.",
									type: "string",
									minLength: 1
								}
							},
							{
								description: "Ignore request (replace with empty module).",
								enum: [false]
							},
							{
								description: "New request.",
								type: "string",
								minLength: 1
							}
						]
					}
				}
			]
		},
		ResolveOptions: {
			description: "Options object for resolving requests.",
			type: "object",
			additionalProperties: false,
			properties: {
				alias: {
					$ref: "#/definitions/ResolveAlias"
				},
				browserField: {
					description:
						"Fields in the description file (usually package.json) which are used to redirect requests inside the module.",
					type: "boolean"
				},
				conditionNames: {
					description: "Condition names for exports field entry point.",
					type: "array",
					items: {
						description: "Condition names for exports field entry point.",
						type: "string"
					}
				},
				extensions: {
					description:
						"Extensions added to the request when trying to find the file.",
					type: "array",
					items: {
						description:
							"Extension added to the request when trying to find the file.",
						type: "string"
					}
				},
				fallback: {
					description: "Redirect module requests when normal resolving fails.",
					oneOf: [
						{
							$ref: "#/definitions/ResolveAlias"
						}
					]
				},
				fullySpecified: {
					description:
						"Treats the request specified by the user as fully specified, meaning no extensions are added and the mainFiles in directories are not resolved (This doesn't affect requests from mainFields, aliasFields or aliases).",
					type: "boolean"
				},
				mainFields: {
					description:
						"Field names from the description file (package.json) which are used to find the default entry point.",
					type: "array",
					items: {
						description:
							"Field name from the description file (package.json) which are used to find the default entry point.",
						anyOf: [
							{
								type: "array",
								items: {
									description:
										"Part of the field path from the description file (package.json) which are used to find the default entry point.",
									type: "string",
									minLength: 1
								}
							},
							{
								type: "string",
								minLength: 1
							}
						]
					}
				},
				mainFiles: {
					description:
						"Filenames used to find the default entry point if there is no description file or main field.",
					type: "array",
					items: {
						description:
							"Filename used to find the default entry point if there is no description file or main field.",
						type: "string",
						minLength: 1
					}
				},
				modules: {
					description: "Folder names or directory paths where to find modules.",
					type: "array",
					items: {
						description: "Folder name or directory path where to find modules.",
						type: "string",
						minLength: 1
					}
				},
				preferRelative: {
					description:
						"Prefer to resolve module requests as relative request and fallback to resolving as module.",
					type: "boolean"
				},
				byDependency: {
					description:
						'Extra resolve options per dependency category. Typical categories are "commonjs", "amd", "esm".',
					type: "object",
					additionalProperties: {
						description: "Options object for resolving requests.",
						oneOf: [
							{
								$ref: "#/definitions/ResolveOptions"
							}
						]
					}
				},
				tsConfigPath: {
					description: "Path to tsconfig.json",
					type: "string"
				},
				exportsFields: {
					description:
						"Fields in the description file (usually package.json) which are used to redirect requests inside the module.",
					type: "array",
					items: {
						description:
							"Field name from the description file (package.json) which are used to find the default entry point.",
						type: "string"
					}
				}
			}
		},
		RuleSetCondition: {
			description: "A condition matcher.",
			anyOf: [
				{
					instanceof: "RegExp"
				},
				{
					type: "string"
				},
				{
					instanceof: "Function"
				},
				{
					$ref: "#/definitions/RuleSetLogicalConditions"
				},
				{
					$ref: "#/definitions/RuleSetConditions"
				}
			]
		},
		RuleSetConditionOrConditions: {
			description: "One or multiple rule conditions.",
			anyOf: [
				{
					$ref: "#/definitions/RuleSetCondition"
				},
				{
					$ref: "#/definitions/RuleSetConditions"
				}
			]
		},
		RuleSetConditions: {
			description: "A list of rule conditions.",
			type: "array",
			items: {
				description: "A rule condition.",
				oneOf: [
					{
						$ref: "#/definitions/RuleSetCondition"
					}
				]
			}
		},
		RuleSetLoader: {
			description: "A loader request.",
			type: "string",
			minLength: 1
		},
		RuleSetLoaderOptions: {
			description: "Options passed to a loader.",
			anyOf: [
				{
					type: "string"
				},
				{
					type: "object"
				}
			]
		},
		RuleSetLogicalConditions: {
			description: "Logic operators used in a condition matcher.",
			type: "object",
			additionalProperties: false,
			properties: {
				and: {
					description: "Logical AND.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditions"
						}
					]
				},
				not: {
					description: "Logical NOT.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetCondition"
						}
					]
				},
				or: {
					description: "Logical OR.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditions"
						}
					]
				}
			}
		},
		RuleSetRule: {
			description:
				"A rule description with conditions and effects for modules.",
			type: "object",
			additionalProperties: false,
			properties: {
				enforce: {
					description: "Enforce this rule as pre or post step.",
					enum: ["pre", "post"]
				},
				exclude: {
					description: "Shortcut for resource.exclude.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				generator: {
					description: "The options for the module generator.",
					type: "object"
				},
				include: {
					description: "Shortcut for resource.include.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				issuer: {
					description:
						"Match the issuer of the module (The module pointing to this module).",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				dependency: {
					description: "Match dependency type.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				descriptionData: {
					description:
						"Match values of properties in the description file (usually package.json).",
					type: "object",
					additionalProperties: {
						$ref: "#/definitions/RuleSetConditionOrConditions"
					}
				},
				oneOf: {
					description: "Only execute the first matching rule in this array.",
					type: "array",
					items: {
						description: "A rule.",
						oneOf: [
							{
								$ref: "#/definitions/RuleSetRule"
							}
						]
					}
				},
				parser: {
					description: "Options for parsing.",
					type: "object",
					additionalProperties: true
				},
				resolve: {
					description: "Options for the resolver.",
					type: "object",
					oneOf: [
						{
							$ref: "#/definitions/ResolveOptions"
						}
					]
				},
				resource: {
					description: "Match the resource path of the module.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				resourceFragment: {
					description: "Match the resource fragment of the module.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				resourceQuery: {
					description: "Match the resource query of the module.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				rules: {
					description:
						"Match and execute these rules when this rule is matched.",
					type: "array",
					items: {
						description: "A rule.",
						oneOf: [
							{
								$ref: "#/definitions/RuleSetRule"
							}
						]
					}
				},
				sideEffects: {
					description: "Flags a module as with or without side effects.",
					type: "boolean"
				},
				test: {
					description: "Shortcut for resource.test.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetConditionOrConditions"
						}
					]
				},
				type: {
					description: "Module type to use for the module.",
					type: "string"
				},
				use: {
					description: "Modifiers applied to the module when rule is matched.",
					oneOf: [
						{
							$ref: "#/definitions/RuleSetUse"
						}
					]
				}
			}
		},
		RuleSetRules: {
			description: "A list of rules.",
			type: "array",
			items: {
				description: "A rule.",
				anyOf: [
					{
						enum: ["..."]
					},
					{
						$ref: "#/definitions/RuleSetRule"
					}
				]
			}
		},
		RuleSetUse: {
			description: "A list of descriptions of loaders applied.",
			anyOf: [
				{
					type: "array",
					items: {
						description: "An use item.",
						oneOf: [
							{
								$ref: "#/definitions/RuleSetUseItem"
							}
						]
					}
				},
				{
					$ref: "#/definitions/RuleSetUseItem"
				}
			]
		},
		RuleSetUseItem: {
			description: "A description of an applied loader.",
			anyOf: [
				{
					type: "object",
					additionalProperties: false,
					properties: {
						loader: {
							description: "Loader name.",
							oneOf: [
								{
									$ref: "#/definitions/RuleSetLoader"
								}
							]
						},
						options: {
							description: "Loader options.",
							oneOf: [
								{
									$ref: "#/definitions/RuleSetLoaderOptions"
								}
							]
						}
					}
				},
				{
					$ref: "#/definitions/RuleSetLoader"
				}
			]
		},
		SnapshotOptions: {
			description:
				"Options affecting how file system snapshots are created and validated.",
			type: "object",
			additionalProperties: false,
			properties: {
				module: {
					description:
						"Options for snapshotting dependencies of modules to determine if they need to be built again.",
					type: "object",
					additionalProperties: false,
					properties: {
						hash: {
							description:
								"Use hashes of the content of the files/directories to determine invalidation.",
							type: "boolean"
						},
						timestamp: {
							description:
								"Use timestamps of the files/directories to determine invalidation.",
							type: "boolean"
						}
					}
				},
				resolve: {
					description:
						"Options for snapshotting dependencies of request resolving to determine if requests need to be re-resolved.",
					type: "object",
					additionalProperties: false,
					properties: {
						hash: {
							description:
								"Use hashes of the content of the files/directories to determine invalidation.",
							type: "boolean"
						},
						timestamp: {
							description:
								"Use timestamps of the files/directories to determine invalidation.",
							type: "boolean"
						}
					}
				}
			}
		},
		StatsOptions: {
			description: "Stats options object.",
			type: "object",
			additionalProperties: true,
			properties: {
				all: {
					description:
						"Fallback value for stats options when an option is not defined (has precedence over local rspack defaults).",
					type: "boolean"
				},
				assets: {
					description: "Add assets information.",
					type: "boolean"
				},
				chunkGroups: {
					description:
						"Display all chunk groups with the corresponding bundles.",
					type: "boolean"
				},
				chunks: {
					description: "Add chunk information.",
					type: "boolean"
				},
				colors: {
					description: "Enables/Disables colorful output.",
					type: "boolean"
				},
				entrypoints: {
					description:
						"Display the entry points with the corresponding bundles.",
					anyOf: [
						{
							enum: ["auto"]
						},
						{
							type: "boolean"
						}
					]
				},
				errors: {
					description: "Add errors.",
					type: "boolean"
				},
				errorsCount: {
					description: "Add errors count.",
					type: "boolean"
				},
				hash: {
					description: "Add the hash of the compilation.",
					type: "boolean"
				},
				modules: {
					description: "Add built modules information.",
					type: "boolean"
				},
				preset: {
					description: "Preset for the default values.",
					anyOf: [
						{
							type: "boolean"
						},
						{
							type: "string"
						}
					]
				},
				publicPath: {
					description: "Add public path information.",
					type: "boolean"
				},
				reasons: {
					description:
						"Add information about the reasons why modules are included.",
					type: "boolean"
				},
				warnings: {
					description: "Add warnings.",
					type: "boolean"
				},
				warningsCount: {
					description: "Add warnings count.",
					type: "boolean"
				},
				outputPath: {
					description: "Add output path information.",
					type: "boolean"
				},
				chunkModules: {
					description: "Add built modules information to chunk information.",
					type: "boolean"
				},
				chunkRelations: {
					description:
						"Add information about parent, children and sibling chunks to chunk information.",
					type: "boolean"
				},
				timings: {
					description: "Add timing information.",
					type: "boolean"
				},
				builtAt: {
					description: "Add built at time information.",
					type: "boolean"
				},
				nestedModules: {
					description:
						"Add information about modules nested in other modules (like with module concatenation).",
					type: "boolean"
				}
			}
		},
		StatsValue: {
			description: "Stats options object or preset name.",
			anyOf: [
				{
					enum: ["none", "errors-only", "errors-warnings", "normal", "verbose"]
				},
				{
					type: "boolean"
				},
				{
					$ref: "#/definitions/StatsOptions"
				}
			]
		},
		StrictModuleErrorHandling: {
			description:
				"Handles error in module loading correctly at a performance cost. This will handle module error compatible with the EcmaScript Modules spec.",
			type: "boolean"
		},
		Target: {
			description:
				"Environment to build for. An array of environments to build for all of them when possible.",
			anyOf: [
				{
					type: "array",
					items: {
						description: "Environment to build for.",
						type: "string",
						minLength: 1
					},
					minItems: 1
				},
				{
					enum: [false]
				},
				{
					type: "string",
					minLength: 1
				}
			]
		},
		TrustedTypes: {
			description: "Use a Trusted Types policy to create urls for chunks.",
			type: "object",
			additionalProperties: false,
			properties: {
				policyName: {
					description:
						"The name of the Trusted Types policy created by webpack to serve bundle chunks.",
					type: "string",
					minLength: 1
				}
			}
		},
		UmdNamedDefine: {
			description:
				"If `output.libraryTarget` is set to umd and `output.library` is set, setting this to true will name the AMD module.",
			type: "boolean"
		},
		UniqueName: {
			description:
				"A unique name of the rspack build to avoid multiple rspack runtimes to conflict when using globals.",
			type: "string",
			minLength: 1
		},
		ChunkLoadingGlobal: {
			description: "The global variable used by rspack for loading of chunks.",
			type: "string",
			minLength: 1
		},
		Watch: {
			description: "Enter watch mode, which rebuilds on file change.",
			type: "boolean"
		},
		WatchOptions: {
			description: "Options for the watcher.",
			type: "object",
			additionalProperties: false,
			properties: {
				aggregateTimeout: {
					description:
						"Delay the rebuilt after the first change. Value is a time in ms.",
					type: "number"
				},
				followSymlinks: {
					description:
						"Resolve symlinks and watch symlink and real file. This is usually not needed as rspack already resolves symlinks ('resolve.symlinks').",
					type: "boolean"
				},
				ignored: {
					description:
						"Ignore some files from watching (glob pattern or regexp).",
					anyOf: [
						{
							type: "array",
							items: {
								description:
									"A glob pattern for files that should be ignored from watching.",
								type: "string",
								minLength: 1
							}
						},
						{
							instanceof: "RegExp"
						},
						{
							description:
								"A single glob pattern for files that should be ignored from watching.",
							type: "string",
							minLength: 1
						}
					]
				},
				poll: {
					description: "Enable polling mode for watching.",
					anyOf: [
						{
							description: "`number`: use polling with specified interval.",
							type: "number"
						},
						{
							description: "`true`: use polling.",
							type: "boolean"
						}
					]
				},
				stdin: {
					description: "Stop watching when stdin stream has ended.",
					type: "boolean"
				}
			}
		},
		RspackPluginFunction: {
			description: "Function acting as plugin.",
			instanceof: "Function"
		},
		RspackPluginInstance: {
			description: "Plugin instance.",
			type: "object",
			additionalProperties: true,
			properties: {
				apply: {
					description: "The run point of the plugin, required method.",
					instanceof: "Function"
				}
			},
			required: ["apply"]
		}
	},
	title: "RspackOptions",
	description: "Options object as provided by the user.",
	type: "object",
	additionalProperties: false,
	properties: {
		cache: {
			$ref: "#/definitions/CacheOptions"
		},
		context: {
			$ref: "#/definitions/Context"
		},
		dependencies: {
			$ref: "#/definitions/Dependencies"
		},
		devServer: {
			$ref: "#/definitions/DevServer"
		},
		devtool: {
			$ref: "#/definitions/DevTool"
		},
		entry: {
			$ref: "#/definitions/Entry"
		},
		experiments: {
			$ref: "#/definitions/Experiments"
		},
		externals: {
			$ref: "#/definitions/Externals"
		},
		externalsType: {
			$ref: "#/definitions/ExternalsType"
		},
		externalsPresets: {
			$ref: "#/definitions/ExternalsPresets"
		},
		infrastructureLogging: {
			$ref: "#/definitions/InfrastructureLogging"
		},
		mode: {
			$ref: "#/definitions/Mode"
		},
		module: {
			$ref: "#/definitions/ModuleOptions"
		},
		name: {
			$ref: "#/definitions/Name"
		},
		node: {
			$ref: "#/definitions/Node"
		},
		optimization: {
			$ref: "#/definitions/Optimization"
		},
		output: {
			$ref: "#/definitions/Output"
		},
		plugins: {
			$ref: "#/definitions/Plugins"
		},
		resolve: {
			$ref: "#/definitions/Resolve"
		},
		snapshot: {
			$ref: "#/definitions/SnapshotOptions"
		},
		stats: {
			$ref: "#/definitions/StatsValue"
		},
		target: {
			$ref: "#/definitions/Target"
		},
		watch: {
			$ref: "#/definitions/Watch"
		},
		watchOptions: {
			$ref: "#/definitions/WatchOptions"
		},
		builtins: {
			description: "Builtins features in rspack",
			type: "object",
			additionalProperties: true
		},
		ignoreWarnings: {
			$ref: "#/definitions/ignoreWarnings"
		}
	}
};
