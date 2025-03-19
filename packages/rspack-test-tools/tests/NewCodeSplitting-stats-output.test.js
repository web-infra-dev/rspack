const path = require("path");
const {
	describeByWalk,
	StatsProcessor,
	BasicCaseCreator,
	ECompilerType
} = require("..");

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new StatsProcessor({
			name,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			snapshotName: "NewCodeSplittingStatsOutput",
			overrideOptions(index, context, options) {
				options.experiments ??= {};
				options.experiments.parallelCodeSplitting ??= true;
				return StatsProcessor.overrideOptions(index, context, options);
			}
		})
	],
	description: () => "should print correct stats for"
});

describeByWalk(
	"new code splitting stats output",
	(name, src, dist) => {
		creator.create(name, src, dist);
	},
	{
		level: 1,
		source: path.resolve(__dirname, "./statsOutputCases"),
		dist: path.resolve(__dirname, `./js/new-code-splitting-stats-output`)
	}
);
