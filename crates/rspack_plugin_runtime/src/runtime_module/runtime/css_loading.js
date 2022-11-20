(function () {
	// object to store loaded and loading chunks
	// undefined = chunk not loaded, null = chunk preloaded/prefetched
	// [resolve, reject, Promise] = chunk loading, 0 = chunk loaded
	var installedChunks = { "runtime~index": 0, index: 0 };

	var uniqueName = "webpack";
	var loadCssChunkData = function (target, link, chunkId) {
		var data,
			token = "",
			token2,
			exports = {},
			exportsWithId = [],
			exportsWithDashes = [],
			i = 0,
			cc = 1;
		try {
			if (!link) link = loadStylesheet(chunkId);
			data = link.sheet.cssRules;
			data = data[data.length - 1].style;
		} catch (e) {
			data = getComputedStyle(document.head);
		}
		data = data.getPropertyValue("--webpack-" + uniqueName + "-" + chunkId);
		if (!data) return [];
		for (; cc; i++) {
			cc = data.charCodeAt(i);
			if (cc == 40) {
				token2 = token;
				token = "";
			} else if (cc == 41) {
				exports[token2.replace(/^_/, "")] = token.replace(/^_/, "");
				token = "";
			} else if (cc == 47 || cc == 37) {
				token = token.replace(/^_/, "");
				exports[token] = token;
				exportsWithId.push(token);
				if (cc == 37) exportsWithDashes.push(token);
				token = "";
			} else if (!cc || cc == 44) {
				token = token.replace(/^_/, "");
				exportsWithId.forEach(
					x => (exports[x] = uniqueName + "-" + token + "-" + exports[x])
				);
				exportsWithDashes.forEach(x => (exports[x] = "--" + exports[x]));
				__webpack_require__.r(exports);
				target[token] = ((exports, module) => {
					module.exports = exports;
				}).bind(null, exports);
				token = "";
				exports = {};
				exportsWithId.length = 0;
			} else if (cc == 92) {
				token += data[++i];
			} else {
				token += data[i];
			}
		}
		installedChunks[chunkId] = 0;
	};
	var loadingAttribute = "data-webpack-loading";
	var loadStylesheet = function (chunkId, url, done) {
		var link,
			needAttach,
			key = "chunk-" + chunkId;

		var links = document.getElementsByTagName("link");
		for (var i = 0; i < links.length; i++) {
			var l = links[i];
			if (
				l.rel == "stylesheet" &&
				(l.href == url ||
					l.getAttribute("href") == url ||
					l.getAttribute("data-webpack") == uniqueName + ":" + key)
			) {
				link = l;
				break;
			}
		}
		if (!done) return link;

		if (!link) {
			needAttach = true;
			link = document.createElement("link");
			link.setAttribute("data-webpack", uniqueName + ":" + key);
			link.setAttribute(loadingAttribute, 1);
			link.rel = "stylesheet";
			link.href = url;
		}
		var onLinkComplete = function (prev, event) {
			link.onerror = link.onload = null;
			link.removeAttribute(loadingAttribute);
			clearTimeout(timeout);
			if (event && event.type != "load") link.parentNode.removeChild(link);
			done(event);
			if (prev) return prev(event);
		};
		if (link.getAttribute(loadingAttribute)) {
			var timeout = setTimeout(
				onLinkComplete.bind(null, undefined, { type: "timeout", target: link }),
				120000
			);
			link.onerror = onLinkComplete.bind(null, link.onerror);
			link.onload = onLinkComplete.bind(null, link.onload);
		} else onLinkComplete(undefined, { type: "load", target: link });

		needAttach && document.head.appendChild(link);
		return link;
	};
	// no initial css

	__webpack_require__.f.css = function (chunkId, promises) {
		// css chunk loading
		var installedChunkData = __webpack_require__.o(installedChunks, chunkId)
			? installedChunks[chunkId]
			: undefined;
		if (installedChunkData !== 0) {
			// 0 means "already installed".

			// a Promise means "currently loading".
			if (installedChunkData) {
				promises.push(installedChunkData[2]);
			} else {
				if ("demo_b_js" == chunkId) {
					// setup Promise in chunk cache
					var promise = new Promise(function (resolve, reject) {
						installedChunkData = installedChunks[chunkId] = [resolve, reject];
					});
					promises.push((installedChunkData[2] = promise));

					// start chunk loading
					var url = __webpack_require__.p + __webpack_require__.k(chunkId);
					// create error before stack unwound to get useful stacktrace later
					var error = new Error();
					var loadingEnded = function (event) {
						if (__webpack_require__.o(installedChunks, chunkId)) {
							installedChunkData = installedChunks[chunkId];
							if (installedChunkData !== 0)
								installedChunks[chunkId] = undefined;
							if (installedChunkData) {
								if (event.type !== "load") {
									var errorType = event && event.type;
									var realSrc = event && event.target && event.target.src;
									error.message =
										"Loading css chunk " +
										chunkId +
										" failed.\n(" +
										errorType +
										": " +
										realSrc +
										")";
									error.name = "ChunkLoadError";
									error.type = errorType;
									error.request = realSrc;
									installedChunkData[1](error);
								} else {
									loadCssChunkData(__webpack_require__.m, link, chunkId);
									installedChunkData[0]();
								}
							}
						}
					};
					var link = loadStylesheet(chunkId, url, loadingEnded);
				} else installedChunks[chunkId] = 0;
			}
		}
	};

	// no hmr
})();
