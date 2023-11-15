var webpackQueues =
	typeof Symbol === "function"
		? Symbol("webpack queues")
		: "__webpack_queues__";
var webpackExports =
	typeof Symbol === "function"
		? Symbol("webpack exports")
		: "__webpack_exports__";
var webpackError =
	typeof Symbol === "function" ? Symbol("webpack error") : "__webpack_error__";
var resolveQueue = function (queue) {
	if (queue && queue.d < 1) {
		queue.d = 1;
		queue.forEach(function (fn) { fn.r--; });
		queue.forEach(function (fn) { fn.r-- ? fn.r++ : fn(); });
	}
}
var wrapDeps = function (deps) {
	return deps.map(function (dep) {
		if (dep !== null && typeof dep === "object") {
			if (dep[webpackQueues]) return dep;
			if (dep.then) {
				var queue = [];
				queue.d = 0;
				dep.then(function (r) {
					obj[webpackExports] = r;
					resolveQueue(queue);
				}, function (e) {
					obj[webpackError] = e;
					resolveQueue(queue);
				});
				var obj = {};
				obj[webpackQueues] = function (fn) { fn(queue); };
				return obj;
			}
		}
		var ret = {};
		ret[webpackQueues] = function () { };
		ret[webpackExports] = dep;
		return ret;
	});
};
__webpack_require__.a = function (module, body, hasAwait) {
	var queue;
	hasAwait && ((queue = []).d = -1);
	var depQueues = new Set();
	var exports = module.exports;
	var currentDeps;
	var outerResolve;
	var reject;
	var promise = new Promise(function (resolve, rej) {
		reject = rej;
		outerResolve = resolve;
	});
	promise[webpackExports] = exports;
	promise[webpackQueues] = function (fn) { queue && fn(queue), depQueues.forEach(fn), promise["catch"](function () { }); };
	module.exports = promise;
	body(function (deps) {
		currentDeps = wrapDeps(deps);
		var fn;
		var getResult = function () {
			return currentDeps.map(function (d) {
				if (d[webpackError]) throw d[webpackError];
				return d[webpackExports];
			});
		}
		var promise = new Promise(function (resolve) {
			fn = function () { resolve(getResult); };
			fn.r = 0;
			var fnQueue = function (q) { q !== queue && !depQueues.has(q) && (depQueues.add(q), q && !q.d && (fn.r++, q.push(fn))); };
			currentDeps.map(function (dep) { dep[webpackQueues](fnQueue); });
		});
		return fn.r ? promise : getResult();
	}, function (err) { (err ? reject(promise[webpackError] = err) : outerResolve(exports)), resolveQueue(queue); });
	queue && queue.d < 0 && (queue.d = 0);
};