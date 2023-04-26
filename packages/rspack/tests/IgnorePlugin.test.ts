// @ts-nocheck
import { Compiler, getNormalizedRspackOptions, rspack } from "../src";
const path = require("path");
const { IgnorePlugin } = require("webpack");

class Plugin implements RspackPluginInstance {
	name = "a";

	apply(compiler: Compiler) {
		// Wait for configuration preset plugions to apply all configure webpack defaults
		compiler.hooks.compilation.tap("a", com => {
			com.normalModuleFactory.hooks.beforeResolve.tap("a", (...args) => {
				console.log(...args);
			});
			com.contextModuleFactory.hooks.beforeResolve.tap("a", (...args) => {
				console.log(...args);
			});
		});
	}
}

describe("Ignore Plugin", () => {
	jest.setTimeout(20000);
	function compile(entry: string, options, callback) {
		const noOutputPath = !options.output || !options.output.path;

		options = getNormalizedRspackOptions(options);

		if (!options.mode) options.mode = "development";
		options.entry = entry;
		options.context = path.join(__dirname, "ignorePlugin");
		if (noOutputPath) options.output.path = "/";
		options.optimization = {
			minimize: false
		};
		options.plugins = [
			new IgnorePlugin({
				checkResource: (resource, request) => {
					if (resource.includes("zh") || resource.includes("globalIndex")) {
						return true;
					}
				}
			})
		];
		options.cache = true;
		const logs = {
			mkdir: [],
			writeFile: []
		};

		const c = rspack(options);
		const files = {};
		c.hooks.compilation.tap("CompilerTest", compilation => {
			compilation.bail = true;
		});
		c.run((err, stats) => {
			if (err) throw err;
			expect(typeof stats).toBe("object");
			const compilation = stats.compilation;
			stats = stats.toJson({
				modules: true,
				reasons: true
			});
			expect(typeof stats).toBe("object");
			expect(stats).toHaveProperty("errors");
			expect(Array.isArray(stats.errors)).toBe(true);
			if (stats.errors.length > 0) {
				expect(stats.errors[0]).toBeInstanceOf(Error);
				throw stats.errors[0];
			}
			stats.logs = logs;
			c.close(err => {
				if (err) return callback(err);
				callback(stats, files, compilation);
			});
		});
	}

	let compiler: Compiler;
	afterEach(callback => {
		if (compiler) {
			compiler.close(callback);
			compiler = undefined;
		} else {
			callback();
		}
	});

	it("should be ignore module", done => {
		const outputDist = "dist/ignorePlugin";
		compile(
			"./index.js",
			{
				output: {
					path: outputDist,
					filename: "index.js"
				}
			},
			(stats, files) => {
				expect(stats.modules).toMatchInlineSnapshot(`
			[
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "./index.js",
			    "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			    "issuerPath": [],
			    "moduleType": "javascript/auto",
			    "name": "./index.js",
			    "size": 77,
			    "type": "module",
			  },
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "./locals/en.js",
			    "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals/en.js",
			    "issuer": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerId": "./locals Sync  recursive ^.*$",
			    "issuerName": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerPath": [
			      {
			        "id": "./index.js",
			        "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			        "name": "./index.js",
			      },
			      {
			        "id": "./locals Sync  recursive ^.*$",
			        "identifier": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			        "name": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			      },
			    ],
			    "moduleType": "javascript/auto",
			    "name": "./locals/en.js",
			    "size": 28,
			    "type": "module",
			  },
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "?9372",
			    "identifier": "missing|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./zh",
			    "issuer": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerId": "./locals Sync  recursive ^.*$",
			    "issuerName": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerPath": [
			      {
			        "id": "./index.js",
			        "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			        "name": "./index.js",
			      },
			      {
			        "id": "./locals Sync  recursive ^.*$",
			        "identifier": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			        "name": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			      },
			    ],
			    "moduleType": "javascript/auto",
			    "name": "/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./zh (missing)",
			    "size": 160,
			    "type": "module",
			  },
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "?c05b",
			    "identifier": "missing|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./zh.js",
			    "issuer": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerId": "./locals Sync  recursive ^.*$",
			    "issuerName": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuerPath": [
			      {
			        "id": "./index.js",
			        "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			        "name": "./index.js",
			      },
			      {
			        "id": "./locals Sync  recursive ^.*$",
			        "identifier": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			        "name": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			      },
			    ],
			    "moduleType": "javascript/auto",
			    "name": "/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./zh.js (missing)",
			    "size": 160,
			    "type": "module",
			  },
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "?fd51",
			    "identifier": "missing|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./globalIndex.js",
			    "issuer": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			    "issuerId": "./index.js",
			    "issuerName": "./index.js",
			    "issuerPath": [
			      {
			        "id": "./index.js",
			        "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			        "name": "./index.js",
			      },
			    ],
			    "moduleType": "javascript/auto",
			    "name": "/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin./globalIndex.js (missing)",
			    "size": 160,
			    "type": "module",
			  },
			  {
			    "chunks": [
			      "main",
			    ],
			    "id": "./locals Sync  recursive ^.*$",
			    "identifier": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "issuer": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			    "issuerId": "./index.js",
			    "issuerName": "./index.js",
			    "issuerPath": [
			      {
			        "id": "./index.js",
			        "identifier": "javascript/auto|/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/index.js",
			        "name": "./index.js",
			      },
			    ],
			    "moduleType": "javascript/auto",
			    "name": "(/Users/xw/self/rspack/packages/rspack/tests/ignorePlugin/locals, None, None,  ContextOptions { mode: Sync, recursive: true, reg_exp: RspackRegex(Regex { cr: CompiledRegex { insns: [StartOfLine, Loop1CharBody { min_iters: 0, max_iters: 18446744073709551615, greedy: true }, MatchAnyExceptLineTerminator, EndOfLine, Goal], start_pred: Arbitrary, loops: 0, groups: 0, named_group_indices: {}, flags: Flags { icase: false, multiline: false, dot_all: false, no_opt: false } } }), reg_str: "^.*$", include: None, exclude: None, category: CommonJS, request: "./locals" })",
			    "size": 160,
			    "type": "module",
			  },
			]
		`);
				done();
			}
		);
	});
});
