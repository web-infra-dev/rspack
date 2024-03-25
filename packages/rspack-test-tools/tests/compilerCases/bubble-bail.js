const store = {
	mkdir: [],
	writeFile: [],
	files: {}
};

module.exports = {
	description: "should compile a single file",
	options(context) {
		return {
			mode: "production",
			entry: "./c",
			context: context.getSource(),
			output: {
				path: "/"
			},
			optimization: {
				minimize: false
			}
		};
	},
	async compiler(_, compiler) {
		compiler.outputFileSystem = {
			// CHANGE: Added support for the `options` parameter to enable recursive directory creation,
			// accommodating Rspack's requirement that differs from webpack's usage
			mkdir(path, options, callback) {
				let recursive = false;
				if (typeof options === "function") {
					callback = options;
				} else if (options) {
					if (options.recursive !== undefined) recursive = options.recursive;
				}
				store.mkdir.push(path);
				if (recursive) {
					callback();
				} else {
					const err = new Error();
					err.code = "EEXIST";
					callback(err);
				}
			},
			writeFile(name, content, callback) {
				store.writeFile.push(name, content);
				store.files[name] = content.toString("utf-8");
				callback();
			},
			stat(path, callback) {
				callback(new Error("ENOENT"));
			}
		};
		compiler.hooks.compilation.tap(
			"CompilerTest",
			compilation => (compilation.bail = true)
		);
	},
	async check(_, __, stats) {
		expect(typeof stats).toBe("object");
		const statsJson = stats.toJson({
			modules: true,
			reasons: true
		});
		expect(typeof statsJson).toBe("object");
		expect(Array.isArray(statsJson.errors)).toBe(true);
		if (statsJson.errors.length > 0) {
			expect(statsJson.errors[0]).toBeInstanceOf(Error);
			throw statsJson.errors[0];
		}
		expect(Object.keys(store.files)).toEqual(["/main.js"]);

		const bundle = store.files["/main.js"];
		expect(bundle).toContain("function __webpack_require__(");
		expect(bundle).toContain("This is a");
		expect(bundle).toContain("This is c");
		expect(bundle).not.toContain("2: function(");
		expect(bundle).not.toContain("window");
		expect(bundle).not.toContain("jsonp");
		expect(bundle).not.toContain("fixtures");
	}
};
