import type { RspackOptionsNormalized } from "@rspack/core";

interface Status {
	isUnloading: boolean;
	currentHash: string;
	previousHash: string[];
}

export function reloadApp(
	{ liveReload, hmr }: RspackOptionsNormalized["devServer"],
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
