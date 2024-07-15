import WebpackDevServer from "webpack-dev-server";

let old: InstanceType<typeof WebpackDevServer>["sendStats"] | undefined;

function restoreDevServerPatch() {
	// @ts-expect-error private API
	WebpackDevServer.prototype.sendStats = old;
}

// Patch webpack-dev-server to prevent it from failing to send stats.
// See https://github.com/web-infra-dev/rspack/pull/4028 for details.
function applyDevServerPatch() {
	if (old) return restoreDevServerPatch;

	// @ts-expect-error private API
	old = WebpackDevServer.prototype.sendStats;

	// @ts-expect-error private API
	WebpackDevServer.prototype.sendStats = function sendStats__rspack_patched(
		// @ts-expect-error
		...args
	) {
		const stats = args[1];

		if (!stats) {
			return;
		}

		return old.apply(this, args);
	};

	return restoreDevServerPatch;
}

export { applyDevServerPatch };
