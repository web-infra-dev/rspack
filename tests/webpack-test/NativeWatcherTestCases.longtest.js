const { describeCases } = require("./WatchTestCases.template");

describeCases({
	name: "NativeWatcherTestCases",
	experiments: {
		nativeWatcher: true,
	}
});
