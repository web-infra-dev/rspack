var inProgress = {};

// var dataWebpackPrefix = "webpack:";
// loadScript function to load a script via script tag
__webpack_require__.l = function loadScript(url, done, key, chunkId) {
	// TODO add this after hash
	// if (inProgress[url]) {
	// 	inProgress[url].push(done);
	// 	return;
	// }
	var script, needAttach;
	if (key !== undefined) {
		var scripts = document.getElementsByTagName("script");
		for (var i = 0; i < scripts.length; i++) {
			var s = scripts[i];
			if (
				s.getAttribute("src") == url
				// || s.getAttribute("data-webpack") == dataWebpackPrefix + key
			) {
				script = s;
				break;
			}
		}
	}
	if (!script) {
		needAttach = true;
		script = document.createElement("script");

		script.charset = "utf-8";
		script.timeout = 120;
		// script.setAttribute("data-webpack", dataWebpackPrefix + key);
		script.src = url;

		if (__CROSS_ORIGIN_LOADING_PLACEHOLDER__ && script.src.indexOf(window.location.origin + '/') !== 0) {
			script.crossOrigin = __CROSS_ORIGIN_LOADING_PLACEHOLDER__;
		}
	}
	inProgress[url] = [done];
	var onScriptComplete = function (prev, event) {
		script.onerror = script.onload = null;
		clearTimeout(timeout);
		var doneFns = inProgress[url];
		delete inProgress[url];
		script.parentNode && script.parentNode.removeChild(script);
		doneFns &&
			doneFns.forEach(function (fn) {
				return fn(event);
			});
		if (prev) return prev(event);
	};
	var timeout = setTimeout(
		onScriptComplete.bind(null, undefined, {
			type: "timeout",
			target: script
		}),
		120000
	);
	script.onerror = onScriptComplete.bind(null, script.onerror);
	script.onload = onScriptComplete.bind(null, script.onload);
	needAttach && document.head.appendChild(script);
};
