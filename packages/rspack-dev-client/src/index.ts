import socket from "./socket";
import createSocketURL from "./createSocketURL";
import parseURL from "./parseURL.js";
import type { Handler } from "./socket";
import reloadApp from "./utils/reloadApp";

const status = {
	currentHash: ""
};
const options = {
	hot: true,
	liveReload: true,
	progress: true,
	overlay: true
};
// TODO: change `options` by the result of `parsedResourceQuery`.

const onSocketMessage: Handler = {
	// TODO: remove data after jsonp
	ok: function (): void {
		reloadApp(options, status);
	},
	close: function (): void {
		console.log("hit close");
	},
	"static-changed": function () {
		// Use it after memoryFileSystem.
		self.location.reload();
	}
};

const parsedResourceQuery = parseURL(location.search);
const socketURL = createSocketURL(parsedResourceQuery as any);

socket(socketURL, onSocketMessage);
