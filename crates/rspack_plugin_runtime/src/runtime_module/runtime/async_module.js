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
var resolveQueue = queue => {
	if (queue && !queue.d) {
		queue.d = 1;
		queue.forEach(fn => fn.r--);
		queue.forEach(fn => (fn.r-- ? fn.r++ : fn()));
	}
};
var wrapDeps = deps =>
	deps.map(dep => {
		if (dep !== null && typeof dep === "object") {
			if (dep[webpackQueues]) return dep;
			if (dep.then) {
				var queue = [];
				queue.d = 0;
				dep.then(
					r => {
						obj[webpackExports] = r;
						resolveQueue(queue);
					},
					e => {
						obj[webpackError] = e;
						resolveQueue(queue);
					}
				);
				var obj = {};
				obj[webpackQueues] = fn => fn(queue);
				return obj;
			}
		}
		var ret = {};
		ret[webpackQueues] = x => {};
		ret[webpackExports] = dep;
		return ret;
	});
__webpack_require__.a = (module, body, hasAwait) => {
	var queue;
	hasAwait && ((queue = []).d = 1);
	var depQueues = new Set();
	var exports = module.exports;
	var currentDeps;
	var outerResolve;
	var reject;
	var promise = new Promise((resolve, rej) => {
		reject = rej;
		outerResolve = resolve;
	});
	promise[webpackExports] = exports;
	promise[webpackQueues] = fn => (
		queue && fn(queue), depQueues.forEach(fn), promise["catch"](x => {})
	);
	module.exports = promise;
	body(
		deps => {
			currentDeps = wrapDeps(deps);
			var fn;
			var getResult = () =>
				currentDeps.map(d => {
					if (d[webpackError]) throw d[webpackError];
					return d[webpackExports];
				});
			var promise = new Promise(resolve => {
				fn = () => resolve(getResult);
				fn.r = 0;
				var fnQueue = q =>
					q !== queue &&
					!depQueues.has(q) &&
					(depQueues.add(q), q && !q.d && (fn.r++, q.push(fn)));
				currentDeps.map(dep => dep[webpackQueues](fnQueue));
			});
			return fn.r ? promise : getResult();
		},
		err => (
			err ? reject((promise[webpackError] = err)) : outerResolve(exports),
			resolveQueue(queue)
		)
	);
	queue && (queue.d = 0);
};
