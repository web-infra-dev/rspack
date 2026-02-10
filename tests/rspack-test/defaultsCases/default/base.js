/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "cache true",
	options: () => ({}),
	diff: (_, defaults) =>
		defaults.toMatchInlineSnapshot(`
			Object {
			  amd: undefined,
			  bail: false,
			  cache: false,
			  context: <TEST_ROOT>,
			  dependencies: undefined,
			  devServer: undefined,
			  devtool: false,
			  entry: Object {
			    main: Object {
			      import: Array [
			        ./src,
			      ],
			    },
			  },
			  experiments: Object {
			    asyncWebAssembly: true,
			    buildHttp: undefined,
			    deferImport: false,
			    futureDefaults: false,
			    useInputFileSystem: false,
			  },
			  externals: undefined,
			  externalsPresets: Object {
			    electron: false,
			    electronMain: false,
			    electronPreload: false,
			    electronRenderer: false,
			    node: false,
			    nwjs: false,
			    web: true,
			  },
			  externalsType: var,
			  ignoreWarnings: undefined,
			  incremental: Object {
			    buildChunkGraph: true,
			    buildModuleGraph: true,
			    chunkAsset: true,
			    chunkIds: true,
			    chunksHashes: true,
			    chunksRuntimeRequirements: true,
			    emitAssets: true,
			    finishModules: true,
			    moduleIds: true,
			    modulesCodegen: true,
			    modulesHashes: true,
			    modulesRuntimeRequirements: true,
			    optimizeDependencies: true,
			    silent: true,
			  },
			  infrastructureLogging: Object {},
			  lazyCompilation: false,
			  loader: Object {
			    environment: Object {
			      arrowFunction: true,
			      asyncFunction: true,
			      bigIntLiteral: true,
			      const: true,
			      destructuring: true,
			      document: true,
			      dynamicImport: undefined,
			      dynamicImportInWorker: undefined,
			      forOf: true,
			      globalThis: undefined,
			      importMetaDirnameAndFilename: undefined,
			      methodShorthand: true,
			      module: undefined,
			      nodePrefixForCoreModules: true,
			      optionalChaining: true,
			      templateLiteral: true,
			    },
			    target: web,
			  },
			  mode: none,
			  module: Object {
			    defaultRules: Array [
			      Object {
			        mimetype: application/node,
			        type: javascript/auto,
			      },
			      Object {
			        test: /\\\\\\.json\\$/i,
			        type: json,
			      },
			      Object {
			        mimetype: application/json,
			        type: json,
			      },
			      Object {
			        resolve: Object {
			          byDependency: Object {
			            esm: Object {
			              fullySpecified: true,
			            },
			          },
			        },
			        test: /\\\\\\.mjs\\$/i,
			        type: javascript/esm,
			      },
			      Object {
			        descriptionData: Object {
			          type: module,
			        },
			        resolve: Object {
			          byDependency: Object {
			            esm: Object {
			              fullySpecified: true,
			            },
			          },
			        },
			        test: /\\\\\\.js\\$/i,
			        type: javascript/esm,
			      },
			      Object {
			        test: /\\\\\\.cjs\\$/i,
			        type: javascript/dynamic,
			      },
			      Object {
			        descriptionData: Object {
			          type: commonjs,
			        },
			        test: /\\\\\\.js\\$/i,
			        type: javascript/dynamic,
			      },
			      Object {
			        mimetype: Object {
			          or: Array [
			            text/javascript,
			            application/javascript,
			          ],
			        },
			        resolve: Object {
			          byDependency: Object {
			            esm: Object {
			              fullySpecified: true,
			            },
			          },
			        },
			        type: javascript/esm,
			      },
			      Object {
			        rules: Array [
			          Object {
			            descriptionData: Object {
			              type: module,
			            },
			            resolve: Object {
			              fullySpecified: true,
			            },
			          },
			        ],
			        test: /\\\\\\.wasm\\$/i,
			        type: webassembly/async,
			      },
			      Object {
			        mimetype: application/wasm,
			        rules: Array [
			          Object {
			            descriptionData: Object {
			              type: module,
			            },
			            resolve: Object {
			              fullySpecified: true,
			            },
			          },
			        ],
			        type: webassembly/async,
			      },
			      Object {
			        dependency: url,
			        oneOf: Array [
			          Object {
			            scheme: /\\^data\\$/,
			            type: asset/inline,
			          },
			          Object {
			            type: asset/resource,
			          },
			        ],
			      },
			      Object {
			        type: json,
			        with: Object {
			          type: json,
			        },
			      },
			      Object {
			        type: asset/source,
			        with: Object {
			          type: text,
			        },
			      },
			      Object {
			        type: asset/bytes,
			        with: Object {
			          type: bytes,
			        },
			      },
			    ],
			    generator: Object {
			      css: Object {
			        esModule: true,
			        exportsOnly: false,
			      },
			      css/auto: Object {
			        esModule: true,
			        exportsConvention: as-is,
			        exportsOnly: false,
			        localIdentName: [fullhash],
			      },
			      css/module: Object {
			        esModule: true,
			        exportsConvention: as-is,
			        exportsOnly: false,
			        localIdentName: [fullhash],
			      },
			      json: Object {
			        JSONParse: true,
			      },
			    },
			    noParse: undefined,
			    parser: Object {
			      asset: Object {
			        dataUrlCondition: Object {
			          maxSize: 8096,
			        },
			      },
			      css: Object {
			        namedExports: true,
			        url: true,
			      },
			      css/auto: Object {
			        namedExports: true,
			        url: true,
			      },
			      css/module: Object {
			        namedExports: true,
			        url: true,
			      },
			      javascript: Object {
			        commonjs: true,
			        deferImport: false,
			        dynamicImportMode: lazy,
			        dynamicImportPrefetch: false,
			        dynamicImportPreload: false,
			        exportsPresence: error,
			        exprContextCritical: true,
			        importDynamic: true,
			        importMeta: true,
			        jsx: false,
			        requireAlias: false,
			        requireAsExpression: true,
			        requireDynamic: true,
			        requireResolve: true,
			        typeReexportsPresence: no-tolerant,
			        unknownContextCritical: true,
			        url: true,
			        worker: Array [
			          ...,
			        ],
			        wrappedContextCritical: false,
			        wrappedContextRegExp: /\\.\\*/,
			      },
			      json: Object {
			        exportsDepth: 9007199254740991,
			      },
			    },
			    rules: Array [],
			  },
			  name: undefined,
			  node: Object {
			    __dirname: warn-mock,
			    __filename: warn-mock,
			    global: warn,
			  },
			  optimization: Object {
			    avoidEntryIife: false,
			    chunkIds: natural,
			    concatenateModules: false,
			    emitOnErrors: true,
			    inlineExports: false,
			    innerGraph: false,
			    mangleExports: false,
			    mergeDuplicateChunks: true,
			    minimize: false,
			    minimizer: Array [
			      SwcJsMinimizerRspackPlugin {
			        _args: Array [],
			        affectedHooks: compilation,
			        name: SwcJsMinimizerRspackPlugin,
			      },
			      LightningCssMinimizerRspackPlugin {
			        _args: Array [],
			        affectedHooks: undefined,
			        name: LightningCssMinimizerRspackPlugin,
			      },
			    ],
			    moduleIds: natural,
			    nodeEnv: false,
			    providedExports: true,
			    realContentHash: false,
			    removeAvailableModules: true,
			    removeEmptyChunks: true,
			    runtimeChunk: false,
			    sideEffects: flag,
			    splitChunks: Object {
			      automaticNameDelimiter: -,
			      cacheGroups: Object {
			        default: Object {
			          idHint: ,
			          minChunks: 2,
			          priority: -20,
			          reuseExistingChunk: true,
			        },
			        defaultVendors: Object {
			          idHint: vendors,
			          priority: -10,
			          reuseExistingChunk: true,
			          test: /\\[\\\\\\\\/\\]node_modules\\[\\\\\\\\/\\]/i,
			        },
			      },
			      chunks: async,
			      defaultSizeTypes: Array [
			        javascript,
			        css,
			        unknown,
			      ],
			      hidePathInfo: false,
			      maxAsyncRequests: Infinity,
			      maxInitialRequests: Infinity,
			      minChunks: 1,
			      minSize: 10000,
			      usedExports: false,
			    },
			    usedExports: false,
			  },
			  output: Object {
			    assetModuleFilename: [hash][ext][query],
			    asyncChunks: true,
			    bundlerInfo: Object {
			      bundler: rspack,
			      force: true,
			      version: $version$,
			    },
			    chunkFilename: [name].js,
			    chunkFormat: array-push,
			    chunkLoadTimeout: 120000,
			    chunkLoading: jsonp,
			    chunkLoadingGlobal: rspackChunk_rspack_tests,
			    clean: false,
			    compareBeforeEmit: true,
			    crossOriginLoading: false,
			    cssChunkFilename: [name].css,
			    cssFilename: [name].css,
			    devtoolFallbackModuleFilenameTemplate: undefined,
			    devtoolModuleFilenameTemplate: undefined,
			    devtoolNamespace: @rspack/tests,
			    enabledChunkLoadingTypes: Array [
			      jsonp,
			      import-scripts,
			    ],
			    enabledLibraryTypes: Array [],
			    enabledWasmLoadingTypes: Array [
			      fetch,
			    ],
			    environment: Object {
			      arrowFunction: true,
			      asyncFunction: true,
			      bigIntLiteral: true,
			      const: true,
			      destructuring: true,
			      document: true,
			      dynamicImport: undefined,
			      dynamicImportInWorker: undefined,
			      forOf: true,
			      globalThis: undefined,
			      importMetaDirnameAndFilename: undefined,
			      methodShorthand: true,
			      module: undefined,
			      nodePrefixForCoreModules: true,
			      optionalChaining: true,
			      templateLiteral: true,
			    },
			    filename: [name].js,
			    globalObject: self,
			    hashDigest: hex,
			    hashDigestLength: 16,
			    hashFunction: xxhash64,
			    hashSalt: undefined,
			    hotUpdateChunkFilename: [id].[fullhash].hot-update.js,
			    hotUpdateGlobal: rspackHotUpdate_rspack_tests,
			    hotUpdateMainFilename: [runtime].[fullhash].hot-update.json,
			    iife: true,
			    importFunctionName: import,
			    importMetaName: import.meta,
			    library: undefined,
			    module: false,
			    path: <TEST_ROOT>/dist,
			    pathinfo: false,
			    publicPath: auto,
			    scriptType: false,
			    sourceMapFilename: [file].map[query],
			    strictModuleErrorHandling: false,
			    trustedTypes: undefined,
			    uniqueName: @rspack/tests,
			    wasmLoading: fetch,
			    webassemblyModuleFilename: [hash].module.wasm,
			    workerChunkLoading: import-scripts,
			    workerPublicPath: ,
			    workerWasmLoading: fetch,
			  },
			  performance: false,
			  plugins: Array [],
			  resolve: Object {
			    aliasFields: Array [],
			    byDependency: Object {
			      amd: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          require,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      commonjs: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          require,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      css-import: Object {
			        conditionNames: Array [
			          webpack,
			          production,
			          style,
			        ],
			        extensions: Array [
			          .css,
			        ],
			        mainFields: Array [
			          style,
			          ...,
			        ],
			        mainFiles: Array [],
			        preferRelative: true,
			      },
			      esm: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          import,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      loader: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          require,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      loaderImport: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          import,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      unknown: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          require,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      url: Object {
			        preferRelative: true,
			      },
			      wasm: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          import,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			      },
			      worker: Object {
			        aliasFields: Array [
			          browser,
			        ],
			        conditionNames: Array [
			          import,
			          module,
			          ...,
			        ],
			        extensions: Array [
			          .js,
			          .json,
			          .wasm,
			        ],
			        mainFields: Array [
			          browser,
			          module,
			          ...,
			        ],
			        preferRelative: true,
			      },
			    },
			    conditionNames: Array [
			      webpack,
			      production,
			      browser,
			    ],
			    exportsFields: Array [
			      exports,
			    ],
			    extensions: Array [],
			    importsFields: Array [
			      imports,
			    ],
			    mainFields: Array [
			      main,
			    ],
			    mainFiles: Array [
			      index,
			    ],
			    modules: Array [
			      node_modules,
			    ],
			    pnp: false,
			    roots: Array [
			      <TEST_ROOT>,
			    ],
			  },
			  resolveLoader: Object {
			    conditionNames: Array [
			      loader,
			      require,
			      node,
			    ],
			    exportsFields: Array [
			      exports,
			    ],
			    extensions: Array [
			      .js,
			    ],
			    mainFields: Array [
			      loader,
			      main,
			    ],
			    mainFiles: Array [
			      index,
			    ],
			  },
			  snapshot: Object {},
			  stats: Object {},
			  target: web,
			  watch: false,
			  watchOptions: Object {},
			}
		`)
};
