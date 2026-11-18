import { createFsFromVolume, Volume } from "memfs";
import { rspack } from "@rspack/core";

const { RawSource } = rspack.sources;

// A large asset makes each read take long enough to overlap the next rebuild,
// which swaps compilations on a native thread.
const SIZE = 128 * 1024 * 1024;
const REBUILDS = 12;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default (() => {
	const unit = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!#";
	const content = unit.repeat(SIZE / unit.length);
	const reads = [];
	return {
		description:
			"should keep a finished build's assets readable while the next rebuild starts",
		options(context) {
			return {
				context: context.getSource(),
				entry: "./d",
				devtool: false,
				plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap("Test", compilation => {
								compilation.hooks.processAssets.tap("Test", () => {
									compilation.emitAsset("big.txt", new RawSource(content));
								});
							});
						}
					}
				]
			};
		},
		async compiler(context, compiler) {
			compiler.outputFileSystem = createFsFromVolume(new Volume());
		},
		async build(context, compiler) {
			await new Promise((resolve, reject) => {
				compiler.hooks.done.tap("Test", stats => {
					if (reads.length === REBUILDS) return;
					// Start the next rebuild, then read this build's asset while it runs.
					compiler.watching.invalidate();
					const asset = stats.compilation.getAsset("big.txt");
					const source = asset && asset.source;
					const value = source ? source.source() : undefined;
					reads.push(
						value === undefined ? "absent" : value === content ? "ok" : "corrupt"
					);
				});
				let builds = 0;
				const watching = compiler.watch({}, err => {
					if (err) return reject(err);
					if (++builds > REBUILDS) watching.close(resolve);
				});
			});
		},
		async check() {
			expect(reads).toHaveLength(REBUILDS);
			expect(reads.filter(read => read === "corrupt")).toEqual([]);
		}
	};
})();
