import socket from "./socket";
import createSocketURL from "./createSocketURL";
import parseURL from "./parseURL.js";
import type { Handler } from "./socket";

// const parsedResourceQuery = parseURL(document.location.toString());
// const socketURL = createSocketURL(parsedResourceQuery as any);

function reloadApp(data: string) {
	// @ts-ignore
	self.__rspack_runtime__.__rspack_require__.hmrM =
		// @ts-ignore
		self.__rspack_runtime__.__rspack_require__.hmrM ||
		(() => {
			return new Promise(resolve => {
				const { uri, content } = JSON.parse(data);
				const update = {
					c: ["main"],
					r: [],
					m: [],
					// TODO: remove this after hash
					updatedModule: {
						uri,
						content: `self["hotUpdate"](
						"main", 
						{ 
							"${uri}": function (module, exports) { ${content} } 
						}
						)`
					}
				};
				resolve(update);
			});
		});
	// @ts-ignore
	self.__rspack_runtime__.hotEmitter.emit("hotUpdate");
}

const onSocketMessage: Handler = {
	// TODO: remove data after jsonp
	ok: function (data): void {
		reloadApp(data);
	},
	close: function (): void {
		console.log("hit close");
	},
	"static-changed": function () {
		self.location.reload();
	}
};

socket(`ws://${location.host}/ws`, onSocketMessage);
