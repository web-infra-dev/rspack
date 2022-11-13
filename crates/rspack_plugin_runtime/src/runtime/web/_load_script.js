(() => {
	var inProgress = {};
	// data-webpack is not used as build has no uniqueName
	// loadScript function to load a script via script tag
	runtime.__rspack_require__.l = (url, done, key, chunkId) => {
		// add this after hash
		// if (inProgress[url]) {
		// 	inProgress[url].push(done);
		// 	return;
		// }
		var script, needAttach;
		if (key !== undefined) {
			var scripts = document.getElementsByTagName("script");
			for (var i = 0; i < scripts.length; i++) {
				var s = scripts[i];
				if (s.getAttribute("src") == url) {
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
			// if (__webpack_require__.nc) {
			// 	script.setAttribute("nonce", __webpack_require__.nc);
			// }

			script.src = url;
		}
		inProgress[url] = [done];
		var onScriptComplete = (prev, event) => {
			// avoid mem leaks in IE.
			script.onerror = script.onload = null;
			clearTimeout(timeout);
			var doneFns = inProgress[url];
			delete inProgress[url];
			script.parentNode && script.parentNode.removeChild(script);
			doneFns && doneFns.forEach(fn => fn(event));
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
})();
