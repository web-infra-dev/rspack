import socket from "./socket";
import createSocketURL from "./createSocketURL";
import parseURL from "webpack-dev-server/client/utils/parseURL";
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

// TODO: should change `location.search` to `__resourceQuery`.
const parsedResourceQuery = parseURL(location.search);
const socketURL = createSocketURL(parsedResourceQuery as any);

socket(socketURL, onSocketMessage);
