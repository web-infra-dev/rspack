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
		// TODO: after memory filesystem
		// self.location.reload();
	},
	update: function (data) {
		const { uri, content } = JSON.parse(data);
		const code = `__rspack_runtime__.installedModules[${JSON.stringify(
			uri
		)}] = function (module, exports, __rspack_require__, __rspack_dynamic_require__) { ${content} }; __rspack_runtime__.invalidate(${JSON.stringify(
			uri
		)})`;
		(0, eval)(code);
	}
};

socket(`ws://${location.host}/ws`, onSocketMessage);
