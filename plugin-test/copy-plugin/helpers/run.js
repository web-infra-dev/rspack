// Ideally we pass in patterns and confirm the resulting assets
const fs = require("fs");
const { rspack } = require("@rspack/core");
const AsyncEventIterator = require("./AsyncEventIterator");

const removeIllegalCharacterForWindows = require("./removeIllegalCharacterForWindows");

const { compile, getCompiler, readAssets } = require("./index");
const { transformWindowPath } = require("./readAssets");

function resolveCopy(copy) {
	if (!copy) {
		return undefined;
	}

	const ret = {
		patterns: []
	};

	ret.patterns = (copy.patterns || []).map(pattern => {
		if (typeof pattern === "string") {
			pattern = { from: pattern };
		}

		pattern.force ??= false;
		pattern.noErrorOnMissing ??= false;
		pattern.priority ??= 0;
		pattern.globOptions ??= {};

		return pattern;
	});

	return ret;
}

/* eslint-disable no-param-reassign */

const isWin = process.platform === "win32";

const ignore = [
	"**/symlink/**/*",
	"**/file-ln.txt",
	"**/directory-ln",
	"**/watch/**/*"
];

function run(opts) {
	return new Promise((resolve, reject) => {
		if (Array.isArray(opts.patterns)) {
			opts.patterns.forEach(pattern => {
				if (pattern.context) {
					// eslint-disable-next-line no-param-reassign
					pattern.context = removeIllegalCharacterForWindows(pattern.context);
				}

				if (typeof pattern !== "string") {
					if (!opts.symlink || isWin) {
						pattern.globOptions = pattern.globOptions || {};
						pattern.globOptions.ignore = [
							...ignore,
							...(pattern.globOptions.ignore || [])
						];
					}
				}
			});
		}

		const compiler =
			opts.compiler ||
			getCompiler({
				plugins: [
					new rspack.CopyRspackPlugin(
						resolveCopy({
							patterns: opts.patterns,
							options: opts.options
						})
					)
				]
			});

		// Execute the functions in series
		return compile(compiler)
			.then(({ stats }) => {
				const { compilation } = stats;

				if (opts.expectedErrors) {
					expect(compilation.errors).toEqual(opts.expectedErrors);
				} else if (compilation.errors.length > 0) {
					throw compilation.errors[0];
				}

				if (opts.expectedWarnings) {
					expect(compilation.warnings).toEqual(opts.expectedWarnings);
				} else if (compilation.warnings.length > 0) {
					throw compilation.warnings[0];
				}

				resolve({ compilation, compiler, stats });
			})
			.catch(reject);
	});
}

function runEmit(opts) {
	return run(opts).then(({ compilation, compiler, stats }) => {
		if (opts.skipAssetsTesting) {
			return;
		}

		if (opts.expectedAssetKeys && opts.expectedAssetKeys.length > 0) {
			expect(
				Object.keys(compilation.assets)
					.filter(a => a !== "main.js")
					.map(transformWindowPath)
					.sort()
			).toEqual(
				opts.expectedAssetKeys.sort().map(removeIllegalCharacterForWindows)
			);
		} else {
			// eslint-disable-next-line no-param-reassign
			delete compilation.assets["main.js"];
			expect(compilation.assets).toEqual({});
		}

		if (opts.expectedAssetContent) {
			// eslint-disable-next-line guard-for-in
			for (const assetName in opts.expectedAssetContent) {
				expect(compilation.assets[assetName]).toBeDefined();

				if (compilation.assets[assetName]) {
					let expectedContent = opts.expectedAssetContent[assetName];
					let compiledContent = readAssets(compiler, stats)[assetName];

					if (!Buffer.isBuffer(expectedContent)) {
						expectedContent = Buffer.from(expectedContent);
					}

					if (!Buffer.isBuffer(compiledContent)) {
						compiledContent = Buffer.from(compiledContent);
					}

					expect(Buffer.compare(expectedContent, compiledContent)).toBe(0);
				}
			}
		}
	});
}

function runForce(opts) {
	// eslint-disable-next-line no-param-reassign
	opts.compiler = getCompiler();

	new PreCopyPlugin({ options: opts }).apply(opts.compiler);

	return runEmit(opts).then(() => { });
}

const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

async function runChange(opts) {
	const compiler = getCompiler();

	new rspack.CopyRspackPlugin(
		resolveCopy({
			patterns: opts.patterns,
			options: opts.options
		})
	).apply(compiler);

	const waitChanges = new AsyncEventIterator();

	// Create two test files
	fs.writeFileSync(opts.newFileLoc1, "file1contents");
	fs.writeFileSync(opts.newFileLoc2, "file2contents");

	const watching = compiler.watch({}, (error, stats) => {
		if (error || stats.hasErrors()) {
			return waitChanges.reject(error || stats.compilation.errors);
		}
		return waitChanges.resolve(readAssets(compiler, stats));
	});

	const assetsBefore = (await waitChanges.next()).value;
	fs.appendFileSync(opts.newFileLoc1, "extra");
	const assetsAfter = (await waitChanges.next()).value;

	await waitChanges.return(); // dispose

	return new Promise(resolve => {
		watching.close(() => {
			const filesForCompare = Object.keys(assetsBefore);
			const changedFiles = [];

			filesForCompare.forEach(file => {
				if (assetsBefore[file] !== assetsAfter[file]) {
					changedFiles.push(file);
				}
			});

			const lastFiles = Object.keys(assetsAfter);

			if (
				opts.expectedAssetKeys &&
				opts.expectedAssetKeys.length > 0 &&
				changedFiles.length > 0
			) {
				expect(lastFiles.sort()).toEqual(
					opts.expectedAssetKeys.sort().map(removeIllegalCharacterForWindows)
				);
			} else {
				expect(lastFiles).toEqual([]);
			}

			resolve(watching);
		});
	}).then(() => {
		fs.unlinkSync(opts.newFileLoc1);
		fs.unlinkSync(opts.newFileLoc2);
	});
}

module.exports = { run, runChange, runEmit, runForce };
