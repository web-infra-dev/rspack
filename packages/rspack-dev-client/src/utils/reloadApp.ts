interface Status {
	// TODO: should use hash.
	currentHash: string;
	prevHash?: string;
}

function reloadApp(
	{ hot, liveReload }: { hot: boolean; liveReload: boolean },
	status: Status
) {
	// if (status.isUnloading) {
	//   return;
	// }

	// const { currentHash, previousHash = "" } = status;
	// const isInitial =
	//   currentHash.indexOf(/** @type {string} */(previousHash)) >= 0;

	// if (isInitial) {
	//   return;
	// }

	/**
	 * @param {Window} rootWindow
	 * @param {number} intervalId
	 */
	function applyReload(rootWindow, intervalId) {
		clearInterval(intervalId);

		console.log("App updated. Reloading...");

		rootWindow.location.reload();
	}

	// TODO: use rspack-dev-server
	const search = self.location.search.toLowerCase();
	const allowToHot = search.indexOf("webpack-dev-server-hot=false") === -1;
	const allowToLiveReload =
		search.indexOf("webpack-dev-server-live-reload=false") === -1;

	if (hot && allowToHot) {
		console.log("App hot update...");

		// @ts-ignore
		self.hotEmitter.emit("hotUpdate");

		// TODO: revert it after hash
		// if (typeof self !== "undefined" && self.window) {
		//   // broadcast update to window
		//   self.postMessage(`webpackHotUpdate${status.currentHash}`, "*");
		// }
	}
	// allow refreshing the page only if liveReload isn't disabled
	else if (liveReload && allowToLiveReload) {
		let rootWindow = self;

		// use parent window for reload (in case we're in an iframe with no valid src)
		const intervalId = self.setInterval(() => {
			if (rootWindow.location.protocol !== "about:") {
				// reload immediately if protocol is valid
				applyReload(rootWindow, intervalId);
			} else {
				// @ts-ignored
				rootWindow = rootWindow.parent;

				if (rootWindow.parent === rootWindow) {
					// if parent equals current window we've reached the root which would continue forever, so trigger a reload anyways
					applyReload(rootWindow, intervalId);
				}
			}
		});
	}
}

export default reloadApp;
