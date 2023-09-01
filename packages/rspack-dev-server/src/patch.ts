import WebpackDevServer from "webpack-dev-server";

let old: InstanceType<typeof WebpackDevServer>["sendStats"] | undefined;

function restoreDevServerPatch() {
	// @ts-expect-error private API
	WebpackDevServer.prototype.sendStats = old;
}

function applyDevServerPatch() {
	if (old) return restoreDevServerPatch;

	// @ts-expect-error private API
	old = WebpackDevServer.prototype.sendStats;

	// @ts-expect-error private API
	WebpackDevServer.prototype.sendStats = function sendStats__rspack_patched(
		...args
	) {
		let stats = args[1];

		if (!stats) {
			return;
		}

		return old.apply(this, args);
	};

	return restoreDevServerPatch;
}

export { applyDevServerPatch };
