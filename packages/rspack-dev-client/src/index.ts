import socket from "./socket";
import createSocketURL from "./createSocketURL";
import parseURL from "./parseURL.js";
import type { Handler } from "./socket";

// const parsedResourceQuery = parseURL(document.location.toString());
// const socketURL = createSocketURL(parsedResourceQuery as any);

const onSocketMessage: Handler = {
	ok: function (): void {
		console.log("hit ok");
	},
	close: function (): void {
		console.log("hit close");
	},
	"static-changed": function () {
		self.location.reload();
	}
};

socket(`ws://${location.host}`, onSocketMessage);
