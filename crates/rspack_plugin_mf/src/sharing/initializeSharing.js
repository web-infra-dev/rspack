var initPromises = {};
var initTokens = {};
__webpack_require__.I = function(name, initScope) {
	if (!initScope) initScope = [];
	// handling circular init calls
	var initToken = initTokens[name];
	if (!initToken) initToken = initTokens[name] = {};
	if (initScope.indexOf(initToken) >= 0) return;
	initScope.push(initToken);
	// only runs once
	if (initPromises[name]) return initPromises[name];
	// creates a new share scope if needed
	if (!__webpack_require__.o(__webpack_require__.S, name))
		__webpack_require__.S[name] = {};
	// runs all init snippets from all modules reachable
	var scope = __webpack_require__.S[name];
	var warn = function (msg) {
		if (typeof console !== "undefined" && console.warn) console.warn(msg);
	};
	var uniqueName = __webpack_require__.initializeSharingData.uniqueName;
	var register = function (name, version, factory, eager) {
		var versions = (scope[name] = scope[name] || {});
		var activeVersion = versions[version];
		if (
			!activeVersion ||
			(!activeVersion.loaded &&
				(!eager != !activeVersion.eager
					? eager
					: uniqueName > activeVersion.from))
		)
			versions[version] = { get: factory, from: uniqueName, eager: !!eager };
	};
	var initExternal = function (id) {
		var handleError = function (err) {
			warn("Initialization of sharing external failed: " + err);
		};
		try {
			var module = __webpack_require__(id);
			if (!module) return;
			var initFn = function (module) {
				return (
					module &&
					module.init &&
					module.init(__webpack_require__.S[name], initScope)
				);
			};
			if (module.then) return promises.push(module.then(initFn, handleError));
			var initResult = initFn(module);
			if (initResult && initResult.then)
				return promises.push(initResult["catch"](handleError));
		} catch (err) {
			handleError(err);
		}
	};
	var promises = [];
	var scopeToSharingDataMapping = __webpack_require__.initializeSharingData.scopeToSharingDataMapping;
	if (scopeToSharingDataMapping[name]) {
		scopeToSharingDataMapping[name].forEach(function (stage) {
			if (typeof stage === "object") register(stage.name, stage.version, stage.factory, stage.eager);
			else initExternal(stage)
		});
	}
	if (!promises.length) return (initPromises[name] = 1);
	return (initPromises[name] = Promise.all(promises).then(function () {
		return (initPromises[name] = 1);
	}));
};
