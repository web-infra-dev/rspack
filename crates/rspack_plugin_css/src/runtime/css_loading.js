var uniqueName = "__UNIQUE_NAME__";
function handleCssComposes(exports, composes) {
  for (var i = 0; i < composes.length; i += 3) {
    var moduleId = composes[i];
    var composeFrom = composes[i + 1];
    var composeVar = composes[i + 2];
    var composedId = __webpack_require__(composeFrom)[composeVar];
    exports[moduleId] = exports[moduleId] + " " + composedId
  }
}
var loadCssChunkData = __CSS_CHUNK_DATA__
var loadingAttribute = "data-webpack-loading";
var loadStylesheet = function (chunkId, url, done, hmr, fetchPriority) {
	var link,
		needAttach,
		key = "chunk-" + chunkId;
	if (!hmr) {
		var links = document.getElementsByTagName("link");
		for (var i = 0; i < links.length; i++) {
			var l = links[i];
			var href = l.getAttribute("href") || l.href;
			if (href && !href.startsWith(__webpack_require__.p)) {
				href =
					__webpack_require__.p + (href.startsWith("/") ? href.slice(1) : href);
			}
			if (
				l.rel == "stylesheet" &&
				((href && href.startsWith(url)) ||
					l.getAttribute("data-webpack") == uniqueName + ":" + key)
			) {
				link = l;
				break;
			}
		}
		if (!done) return link;
	}
	if (!link) {
		needAttach = true;
		link = document.createElement("link");
		if (__webpack_require__.nc) {
			link.setAttribute("nonce", __webpack_require__.nc);
		}
		link.setAttribute("data-webpack", uniqueName + ":" + key);
		if (fetchPriority) {
			link.setAttribute("fetchpriority", fetchPriority);
		}
		link.setAttribute(loadingAttribute, 1);
		link.rel = "stylesheet";
		link.href = url;

		__CROSS_ORIGIN_LOADING_PLACEHOLDER__
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
			__CHUNK_LOAD_TIMEOUT_PLACEHOLDER__
		);
		link.onerror = onLinkComplete.bind(null, link.onerror);
		link.onload = onLinkComplete.bind(null, link.onload);
	} else onLinkComplete(undefined, { type: "load", target: link });
	hmr ? document.head.insertBefore(link, hmr) : needAttach && document.head.appendChild(link);
	return link;
};
__INITIAL_CSS_CHUNK_DATA__
