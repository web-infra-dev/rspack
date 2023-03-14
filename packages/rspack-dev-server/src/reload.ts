/**
 * The following code is modified based on
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/client-src/utils/reloadApp.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack-dev-server/blob/b0f15ace0123c125d5870609ef4691c141a6d187/LICENSE
 */
import type { RspackOptionsNormalized } from "@rspack/core";

interface Status {
	isUnloading: boolean;
	currentHash: string;
	previousHash: string[];
}

export function reloadApp(
	{ liveReload, hot }: RspackOptionsNormalized["devServer"],
	status: Status
) {
	if (status.isUnloading) {
		return;
	}

	function applyReload(rootWindow: Window, intervalId: number) {
		clearInterval(intervalId);
		console.log("App update, Reloading...");
		rootWindow.location.reload();
	}

	if (liveReload) {
		let rootWindow = self;
		const intervalId = self.setInterval(() => {
			if (rootWindow.location.protocol !== "about:") {
				applyReload(rootWindow, intervalId);
			}
		});
	}
}
