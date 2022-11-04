var inProgress = {};
function load_script(url, done, key) {
	var dataWebpackPrefix = "rspack-test:";
	if (inProgress[url]) {
		inProgress[url].push(done);
		return;
	}
	var script, needAttach;
	if (key !== undefined) {
		var scripts = document.getElementsByTagName("script");
		for (var i = 0; i < scripts.length; i++) {
			var s = scripts[i];
			if (
				s.getAttribute("src") == url ||
				s.getAttribute("data-rspack") == dataWebpackPrefix + key
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
		script.setAttribute("data-rspack", dataWebpackPrefix + key);
		script.src = url;
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
		onScriptComplete.bind(null, undefined, { type: "timeout", target: script }),
		120000
	);
	script.onerror = onScriptComplete.bind(null, script.onerror);
	script.onload = onScriptComplete.bind(null, script.onload);
	needAttach && document.head.appendChild(script);
}

function __rspack_load_dynamic_js__(chunkId, promises) {
	var runtime = this;
	var installedChunkData = this.checkById(this.installedChunks, chunkId)
		? this.installedChunks[chunkId]
		: undefined;
	if (installedChunkData !== 0) {
		if (installedChunkData) {
			promises.push(installedChunkData[2]);
		} else {
			var promise = new Promise(
				function (resolve, reject) {
					installedChunkData = this.installedChunks[chunkId] = [
						resolve,
						reject
					];
				}.bind(this)
			);
			promises.push((installedChunkData[2] = promise));
			var url =
				this.publicPath + this.__rspack_get_dynamic_chunk_url__(chunkId, "js");
			var error = new Error();
			var loadingEnded = function (event) {
				if (runtime.checkById(runtime.installedChunks, chunkId)) {
					installedChunkData = runtime.installedChunks[chunkId];
					if (installedChunkData !== 0)
						runtime.installedChunks[chunkId] = undefined;
					if (installedChunkData) {
						var errorType =
							event && (event.type === "load" ? "missing" : event.type);
						var realSrc = event && event.target && event.target.src;
						error.message =
							"Loading chunk " +
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
					}
				}
			};
			load_script(url, loadingEnded, "chunk-" + chunkId);
		}
	}
}

// mount load dynamic js
(function () {
	runtime.__rspack_load_dynamic_js__ = __rspack_load_dynamic_js__;
})();
