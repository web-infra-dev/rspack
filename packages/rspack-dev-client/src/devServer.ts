// copy from webpack/hot/dev-server
// @ts-ignored
// @ts-nocheck
if (module.hot) {
	// var lastHash;
	var upToDate = function upToDate() {
		// TODO: should use hash.
		// return lastHash.indexOf(__webpack_hash__) >= 0;
		return false;
	};
	var log = function (_level, info) {
		console.log(info);
	};
	log.formatError = function (err) {
		var message = err.message;
		var stack = err.stack;
		if (!stack) {
			return message;
		} else if (stack.indexOf(message) < 0) {
			return message + "\n" + stack;
		} else {
			return stack;
		}
	};
	var check = function check() {
		module.hot
			.check(true)
			.then(function (updatedModules) {
				if (!updatedModules) {
					log("warning", "[HMR] Cannot find update. Need to do a full reload!");
					log(
						"warning",
						"[HMR] (Probably because of restarting the webpack-dev-server)"
					);
					window.location.reload();
					return;
				}

				// TODO: add this after hash
				// if (!upToDate()) {
				// 	// check();
				// }

				// require("./log-apply-result")(updatedModules, updatedModules);

				if (upToDate()) {
					log("info", "[HMR] App is up to date.");
				}
			})
			.catch(function (err) {
				var status = module.hot.status();
				if (["abort", "fail"].indexOf(status) >= 0) {
					log(
						"warning",
						"[HMR] Cannot apply update. Need to do a full reload!"
					);
					log("warning", "[HMR] " + log.formatError(err));
					window.location.reload();
				} else {
					log("warning", "[HMR] Update failed: " + log.formatError(err));
				}
			});
	};
	self.hotEmitter = self.hotEmitter || require("./emitter");
	self.hotEmitter.on("hotUpdate", function (_currentHash) {
		// TODO: should use hash
		// lastHash = currentHash;
		if (!upToDate() && module.hot.status() === "idle") {
			log("info", "[HMR] Checking for updates on the server...");
			check();
		}
	});

	log("info", "[HMR] Waiting for update signal from WDS...");
} else {
	throw new Error("[HMR] Hot Module Replacement is disabled.");
}
