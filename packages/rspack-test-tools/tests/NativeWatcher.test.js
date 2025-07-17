const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);
const exec = require("child_process").execSync;

if (process.platform == "linux") {
	const r1 = exec(`cat /proc/sys/fs/inotify/max_user_watches`);
	const r2 = exec("cat /proc/sys/fs/inotify/max_user_instances");

	console.info(`max_user_watches=${r1.toString()}`);
	console.info(`max_user_instances=${r2.toString()}`);
}

describeByWalk(
	__filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(__dirname, `./watchCases`)
	}
);
