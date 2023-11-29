// @ts-nocheck

if (__webpack_require__.MF) {
	var parseRange = function (str) {
		const splitAndConvert = function (str) {
			return str
				.split(".")
				.map(item => (item !== "NaN" && `${+item}` === item ? +item : item));
		};
		// see https://docs.npmjs.com/misc/semver#range-grammar for grammar
		const parsePartial = function (str) {
			const match = /^([^-+]+)?(?:-([^+]+))?(?:\+(.+))?$/.exec(str);
			const ver = match[1] ? [0, ...splitAndConvert(match[1])] : [0];
			if (match[2]) {
				ver.length++;
				ver.push.apply(ver, splitAndConvert(match[2]));
			}

			// remove trailing any matchers
			let last = ver[ver.length - 1];
			while (
				ver.length &&
				(last === undefined || /^[*xX]$/.test(/** @type {string} */ (last)))
			) {
				ver.pop();
				last = ver[ver.length - 1];
			}

			return ver;
		};
		const toFixed = range => {
			if (range.length === 1) {
				// Special case for "*" is "x.x.x" instead of "="
				return [0];
			} else if (range.length === 2) {
				// Special case for "1" is "1.x.x" instead of "=1"
				return [1, ...range.slice(1)];
			} else if (range.length === 3) {
				// Special case for "1.2" is "1.2.x" instead of "=1.2"
				return [2, ...range.slice(1)];
			} else {
				return [range.length, ...range.slice(1)];
			}
		};
		const negate = range => {
			return [-range[0] - 1, ...range.slice(1)];
		};
		const parseSimple = str => {
			// simple       ::= primitive | partial | tilde | caret
			// primitive    ::= ( '<' | '>' | '>=' | '<=' | '=' | '!' ) ( ' ' ) * partial
			// tilde        ::= '~' ( ' ' ) * partial
			// caret        ::= '^' ( ' ' ) * partial
			const match = /^(\^|~|<=|<|>=|>|=|v|!)/.exec(str);
			const start = match ? match[0] : "";
			const remainder = parsePartial(
				start.length ? str.slice(start.length).trim() : str.trim()
			);
			switch (start) {
				case "^":
					if (remainder.length > 1 && remainder[1] === 0) {
						if (remainder.length > 2 && remainder[2] === 0) {
							return [3, ...remainder.slice(1)];
						}
						return [2, ...remainder.slice(1)];
					}
					return [1, ...remainder.slice(1)];
				case "~":
					return [2, ...remainder.slice(1)];
				case ">=":
					return remainder;
				case "=":
				case "v":
				case "":
					return toFixed(remainder);
				case "<":
					return negate(remainder);
				case ">": {
					// and( >=, not( = ) ) => >=, =, not, and
					const fixed = toFixed(remainder);
					// eslint-disable-next-line no-sparse-arrays
					return [, fixed, 0, remainder, 2];
				}
				case "<=":
					// or( <, = ) => <, =, or
					// eslint-disable-next-line no-sparse-arrays
					return [, toFixed(remainder), negate(remainder), 1];
				case "!": {
					// not =
					const fixed = toFixed(remainder);
					// eslint-disable-next-line no-sparse-arrays
					return [, fixed, 0];
				}
				default:
					throw new Error("Unexpected start value");
			}
		};
		const combine = (items, fn) => {
			if (items.length === 1) return items[0];
			const arr = [];
			for (const item of items.slice().reverse()) {
				if (0 in item) {
					arr.push(item);
				} else {
					arr.push(...item.slice(1));
				}
			}
			// eslint-disable-next-line no-sparse-arrays
			return [, ...arr, ...items.slice(1).map(() => fn)];
		};
		const parseRange = str => {
			// range      ::= hyphen | simple ( ' ' ( ' ' ) * simple ) * | ''
			// hyphen     ::= partial ( ' ' ) * ' - ' ( ' ' ) * partial
			const items = str.split(/\s+-\s+/);
			if (items.length === 1) {
				const items = str
					.trim()
					.split(/(?<=[-0-9A-Za-z])\s+/g)
					.map(parseSimple);
				return combine(items, 2);
			}
			const a = parsePartial(items[0]);
			const b = parsePartial(items[1]);
			// >=a <=b => and( >=a, or( <b, =b ) ) => >=a, <b, =b, or, and
			// eslint-disable-next-line no-sparse-arrays
			return [, toFixed(b), negate(b), 1, a, 2];
		};
		const parseLogicalOr = str => {
			// range-set  ::= range ( logical-or range ) *
			// logical-or ::= ( ' ' ) * '||' ( ' ' ) *
			const items = str.split(/\s*\|\|\s*/).map(parseRange);
			return combine(items, 1);
		};
		return parseLogicalOr(str);
	};
	var parseVersion = function (str) {
		var p = function (p) {
				return p.split(".").map(function (p) {
					return +p == p ? +p : p;
				});
			},
			n = /^([^-+]+)?(?:-([^+]+))?(?:\+(.+))?$/.exec(str),
			r = n[1] ? p(n[1]) : [];
		return (
			n[2] && (r.length++, r.push.apply(r, p(n[2]))),
			n[3] && (r.push([]), r.push.apply(r, p(n[3]))),
			r
		);
	};
	var versionLt = function (a, b) {
		(a = parseVersion(a)), (b = parseVersion(b));
		for (var r = 0; ; ) {
			if (r >= a.length) return r < b.length && "u" != (typeof b[r])[0];
			var e = a[r],
				n = (typeof e)[0];
			if (r >= b.length) return "u" == n;
			var t = b[r],
				f = (typeof t)[0];
			if (n != f) return ("o" == n && "n" == f) || "s" == f || "u" == n;
			if ("o" != n && "u" != n && e != t) return e < t;
			r++;
		}
	};
	var rangeToString = function (range) {
		var r = range[0],
			n = "";
		if (1 === range.length) return "*";
		if (r + 0.5) {
			n +=
				0 == r
					? ">="
					: -1 == r
					? "<"
					: 1 == r
					? "^"
					: 2 == r
					? "~"
					: r > 0
					? "="
					: "!=";
			for (var e = 1, a = 1; a < range.length; a++) {
				e--,
					(n +=
						"u" == (typeof (t = range[a]))[0]
							? "-"
							: (e > 0 ? "." : "") + ((e = 2), t));
			}
			return n;
		}
		var g = [];
		for (a = 1; a < range.length; a++) {
			var t = range[a];
			g.push(
				0 === t
					? "not(" + o() + ")"
					: 1 === t
					? "(" + o() + " || " + o() + ")"
					: 2 === t
					? g.pop() + " " + g.pop()
					: rangeToString(t)
			);
		}
		return o();
		function o() {
			return g.pop().replace(/^\((.+)\)$/, "$1");
		}
	};
	var satisfy = function (range, version) {
		if (0 in range) {
			version = parseVersion(version);
			var e = range[0],
				r = e < 0;
			r && (e = -e - 1);
			for (var n = 0, i = 1, a = !0; ; i++, n++) {
				var f,
					s,
					g = i < range.length ? (typeof range[i])[0] : "";
				if (n >= version.length || "o" == (s = (typeof (f = version[n]))[0]))
					return !a || ("u" == g ? i > e && !r : ("" == g) != r);
				if ("u" == s) {
					if (!a || "u" != g) return !1;
				} else if (a)
					if (g == s)
						if (i <= e) {
							if (f != range[i]) return !1;
						} else {
							if (r ? f > range[i] : f < range[i]) return !1;
							f != range[i] && (a = !1);
						}
					else if ("s" != g && "n" != g) {
						if (r || i <= e) return !1;
						(a = !1), i--;
					} else {
						if (i <= e || s < g != r) return !1;
						a = !1;
					}
				else "s" != g && "n" != g && ((a = !1), i--);
			}
		}
		var t = [],
			o = t.pop.bind(t);
		for (n = 1; n < range.length; n++) {
			var u = range[n];
			t.push(
				1 == u ? o() | o() : 2 == u ? o() & o() : u ? satisfy(u, version) : !o()
			);
		}
		return !!o();
	};
	var ensureExistence = function (scopeName, key) {
		var scope = __webpack_require__.S[scopeName];
		if (!scope || !__webpack_require__.o(scope, key))
			throw new Error(
				"Shared module " + key + " doesn't exist in shared scope " + scopeName
			);
		return scope;
	};
	var findVersion = function (scope, key) {
		var versions = scope[key];
		var key = Object.keys(versions).reduce(function (a, b) {
			return !a || versionLt(a, b) ? b : a;
		}, 0);
		return key && versions[key];
	};
	var findSingletonVersionKey = function (scope, key) {
		var versions = scope[key];
		return Object.keys(versions).reduce(function (a, b) {
			return !a || (!versions[a].loaded && versionLt(a, b)) ? b : a;
		}, 0);
	};
	var getInvalidSingletonVersionMessage = function (
		scope,
		key,
		version,
		requiredVersion
	) {
		return (
			"Unsatisfied version " +
			version +
			" from " +
			(version && scope[key][version].from) +
			" of shared singleton module " +
			key +
			" (required " +
			rangeToString(requiredVersion) +
			")"
		);
	};
	var getSingleton = function (scope, scopeName, key, requiredVersion) {
		var version = findSingletonVersionKey(scope, key);
		return get(scope[key][version]);
	};
	var getSingletonVersion = function (scope, scopeName, key, requiredVersion) {
		var version = findSingletonVersionKey(scope, key);
		if (!satisfy(requiredVersion, version))
			warn(
				getInvalidSingletonVersionMessage(scope, key, version, requiredVersion)
			);
		return get(scope[key][version]);
	};
	var getStrictSingletonVersion = function (
		scope,
		scopeName,
		key,
		requiredVersion
	) {
		var version = findSingletonVersionKey(scope, key);
		if (!satisfy(requiredVersion, version))
			throw new Error(
				getInvalidSingletonVersionMessage(scope, key, version, requiredVersion)
			);
		return get(scope[key][version]);
	};
	var findValidVersion = function (scope, key, requiredVersion) {
		var versions = scope[key];
		var key = Object.keys(versions).reduce(function (a, b) {
			if (!satisfy(requiredVersion, b)) return a;
			return !a || versionLt(a, b) ? b : a;
		}, 0);
		return key && versions[key];
	};
	var getInvalidVersionMessage = function (
		scope,
		scopeName,
		key,
		requiredVersion
	) {
		var versions = scope[key];
		return (
			"No satisfying version (" +
			rangeToString(requiredVersion) +
			") of shared module " +
			key +
			" found in shared scope " +
			scopeName +
			".\n" +
			"Available versions: " +
			Object.keys(versions)
				.map(function (key) {
					return key + " from " + versions[key].from;
				})
				.join(", ")
		);
	};
	var getValidVersion = function (scope, scopeName, key, requiredVersion) {
		var entry = findValidVersion(scope, key, requiredVersion);
		if (entry) return get(entry);
		throw new Error(
			getInvalidVersionMessage(scope, scopeName, key, requiredVersion)
		);
	};
	var warn = function (msg) {
		if (typeof console !== "undefined" && console.warn) console.warn(msg);
	};
	var warnInvalidVersion = function (scope, scopeName, key, requiredVersion) {
		warn(getInvalidVersionMessage(scope, scopeName, key, requiredVersion));
	};
	var get = function (entry) {
		entry.loaded = 1;
		return entry.get();
	};
	var init = function (fn) {
		return function (scopeName, a, b, c) {
			var promise = __webpack_require__.I(scopeName);
			if (promise && promise.then)
				return promise.then(
					fn.bind(fn, scopeName, __webpack_require__.S[scopeName], a, b, c)
				);
			return fn(scopeName, __webpack_require__.S[scopeName], a, b, c);
		};
	};

	var load = /*#__PURE__*/ init(function (scopeName, scope, key) {
		ensureExistence(scopeName, key);
		return get(findVersion(scope, key));
	});
	var loadFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		fallback
	) {
		return scope && __webpack_require__.o(scope, key)
			? get(findVersion(scope, key))
			: fallback();
	});
	var loadVersionCheck = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version
	) {
		ensureExistence(scopeName, key);
		return get(
			findValidVersion(scope, key, version) ||
				warnInvalidVersion(scope, scopeName, key, version) ||
				findVersion(scope, key)
		);
	});
	var loadSingleton = /*#__PURE__*/ init(function (scopeName, scope, key) {
		ensureExistence(scopeName, key);
		return getSingleton(scope, scopeName, key);
	});
	var loadSingletonVersionCheck = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version
	) {
		ensureExistence(scopeName, key);
		return getSingletonVersion(scope, scopeName, key, version);
	});
	var loadStrictVersionCheck = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version
	) {
		ensureExistence(scopeName, key);
		return getValidVersion(scope, scopeName, key, version);
	});
	var loadStrictSingletonVersionCheck = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version
	) {
		ensureExistence(scopeName, key);
		return getStrictSingletonVersion(scope, scopeName, key, version);
	});
	var loadVersionCheckFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version,
		fallback
	) {
		if (!scope || !__webpack_require__.o(scope, key)) return fallback();
		return get(
			findValidVersion(scope, key, version) ||
				warnInvalidVersion(scope, scopeName, key, version) ||
				findVersion(scope, key)
		);
	});
	var loadSingletonFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		fallback
	) {
		if (!scope || !__webpack_require__.o(scope, key)) return fallback();
		return getSingleton(scope, scopeName, key);
	});
	var loadSingletonVersionCheckFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version,
		fallback
	) {
		if (!scope || !__webpack_require__.o(scope, key)) return fallback();
		return getSingletonVersion(scope, scopeName, key, version);
	});
	var loadStrictVersionCheckFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version,
		fallback
	) {
		var entry =
			scope &&
			__webpack_require__.o(scope, key) &&
			findValidVersion(scope, key, version);
		return entry ? get(entry) : fallback();
	});
	var loadStrictSingletonVersionCheckFallback = /*#__PURE__*/ init(function (
		scopeName,
		scope,
		key,
		version,
		fallback
	) {
		if (!scope || !__webpack_require__.o(scope, key)) return fallback();
		return getStrictSingletonVersion(scope, scopeName, key, version);
	});
	var resolveHandler = function (data) {
		var strict = false;
		var singleton = false;
		var versionCheck = false;
		var fallback = false;
		var args = [data.shareScope, data.shareKey];
		if (data.requiredVersion) {
			if (data.strictVersion) strict = true;
			if (data.singleton) singleton = true;
			args.push(parseRange(data.requiredVersion));
			versionCheck = true;
		} else if (data.singleton) singleton = true;
		if (data.fallback) {
			fallback = true;
			args.push(data.fallback);
		}
		if (strict && singleton && versionCheck && fallback)
			return function () {
				return loadStrictSingletonVersionCheckFallback.apply(null, args);
			};
		if (strict && versionCheck && fallback)
			return function () {
				return loadStrictVersionCheckFallback.apply(null, args);
			};
		if (singleton && versionCheck && fallback)
			return function () {
				return loadSingletonVersionCheckFallback.apply(null, args);
			};
		if (strict && singleton && versionCheck)
			return function () {
				return loadStrictSingletonVersionCheck.apply(null, args);
			};
		if (singleton && fallback)
			return function () {
				return loadSingletonFallback.apply(null, args);
			};
		if (versionCheck && fallback)
			return function () {
				return loadVersionCheckFallback.apply(null, args);
			};
		if (strict && versionCheck)
			return function () {
				return loadStrictVersionCheck.apply(null, args);
			};
		if (singleton && versionCheck)
			return function () {
				return loadSingletonVersionCheck.apply(null, args);
			};
		if (singleton)
			return function () {
				return loadSingleton.apply(null, args);
			};
		if (versionCheck)
			return function () {
				return loadVersionCheck.apply(null, args);
			};
		if (fallback)
			return function () {
				return loadFallback.apply(null, args);
			};
		return function () {
			return load.apply(null, args);
		};
	};
	var installedModules = {};
	__webpack_require__.MF.consumes = function (data) {
		var chunkId = data.chunkId,
			promises = data.promises,
			chunkMapping = data.chunkMapping,
			moduleToConsumeDataMapping = data.moduleToConsumeDataMapping;
		if (__webpack_require__.o(chunkMapping, chunkId)) {
			chunkMapping[chunkId].forEach(function (id) {
				if (__webpack_require__.o(installedModules, id))
					return promises.push(installedModules[id]);
				var onFactory = function (factory) {
					installedModules[id] = 0;
					__webpack_require__.m[id] = function (module) {
						delete __webpack_require__.c[id];
						module.exports = factory();
					};
				};
				var onError = function (error) {
					delete installedModules[id];
					__webpack_require__.m[id] = function (module) {
						delete __webpack_require__.c[id];
						throw error;
					};
				};
				try {
					var promise = resolveHandler(moduleToConsumeDataMapping[id])();
					if (promise.then) {
						promises.push(
							(installedModules[id] = promise.then(onFactory)["catch"](onError))
						);
					} else onFactory(promise);
				} catch (e) {
					onError(e);
				}
			});
		}
	};
	__webpack_require__.MF.initialConsumes = function (data) {
		var initialConsumes = data.initialConsumesData,
			moduleToConsumeDataMapping = data.moduleToConsumeDataMapping;
		if (initialConsumes) {
			initialConsumes.forEach(function (id) {
				__webpack_require__.m[id] = function (module) {
					// Handle case when module is used sync
					installedModules[id] = 0;
					delete __webpack_require__.c[id];
					var factory = resolveHandler(moduleToConsumeDataMapping[id])();
					if (typeof factory !== "function")
						throw new Error(
							"Shared module is not available for eager consumption: " + id
						);
					module.exports = factory();
				};
			});
		}
	};
	if (__webpack_require__.MF.initialConsumesData) {
		__webpack_require__.MF.initialConsumes({
			initialConsumesData: __webpack_require__.MF.initialConsumesData,
			moduleToConsumeDataMapping:
				__webpack_require__.MF.moduleToConsumeDataMapping
		});
	}
}
