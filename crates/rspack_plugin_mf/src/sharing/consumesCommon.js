var splitAndConvert = function(str) {
  return str.split(".").map(function(item) {
    return +item == item ? +item : item;
  });
};
var parseRange = function(str) {
  // see https://docs.npmjs.com/misc/semver#range-grammar for grammar
  var parsePartial = function(str) {
    var match = /^([^-+]+)?(?:-([^+]+))?(?:\+(.+))?$/.exec(str);
    var ver = match[1] ? [0].concat(splitAndConvert(match[1])) : [0];
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
  var toFixed = function(range) {
    if (range.length === 1) {
      // Special case for "*" is "x.x.x" instead of "="
      return [0];
    } else if (range.length === 2) {
      // Special case for "1" is "1.x.x" instead of "=1"
      return [1].concat(range.slice(1));
    } else if (range.length === 3) {
      // Special case for "1.2" is "1.2.x" instead of "=1.2"
      return [2].concat(range.slice(1));
    } else {
      return [range.length].concat(range.slice(1));
    }
  };
  var negate = function(range) {
    return [-range[0] - 1].concat(range.slice(1));
  };
  var parseSimple = function(str) {
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
            return [3].concat(remainder.slice(1));
          }
          return [2].concat(remainder.slice(1));
        }
        return [1].concat(remainder.slice(1));
      case "~":
        return [2].concat(remainder.slice(1));
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
        return [, fixed, 0, remainder, 2];
      }
      case "<=":
        // or( <, = ) => <, =, or
        return [, toFixed(remainder), negate(remainder), 1];
      case "!": {
        // not =
        const fixed = toFixed(remainder);
        return [, fixed, 0];
      }
      default:
        throw new Error("Unexpected start value");
    }
  };
  var combine = function(items, fn) {
    if (items.length === 1) return items[0];
    const arr = [];
    for (const item of items.slice().reverse()) {
      if (0 in item) {
        arr.push(item);
      } else {
        arr.push.apply(arr, item.slice(1));
      }
    }
    return [,].concat(arr, items.slice(1).map(() => fn));
  };
  var parseRange = function(str) {
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
    return [, toFixed(b), negate(b), 1, a, 2];
  };
  var parseLogicalOr = function(str) {
    // range-set  ::= range ( logical-or range ) *
    // logical-or ::= ( ' ' ) * '||' ( ' ' ) *
    const items = str.split(/\s*\|\|\s*/).map(parseRange);
    return combine(items, 1);
  };
  return parseLogicalOr(str);
};
var parseVersion = function(str) {
	var match = /^([^-+]+)?(?:-([^+]+))?(?:\+(.+))?$/.exec(str);
	/** @type {(string|number|undefined|[])[]} */
	var ver = match[1] ? splitAndConvert(match[1]) : [];
	if (match[2]) {
		ver.length++;
		ver.push.apply(ver, splitAndConvert(match[2]));
	}
	if (match[3]) {
		ver.push([]);
		ver.push.apply(ver, splitAndConvert(match[3]));
	}
	return ver;
}
var versionLt = function(a, b) {
	a = parseVersion(a);
	b = parseVersion(b);
	var i = 0;
	for (;;) {
		// a       b  EOA     object  undefined  number  string
		// EOA        a == b  a < b   b < a      a < b   a < b
		// object     b < a   (0)     b < a      a < b   a < b
		// undefined  a < b   a < b   (0)        a < b   a < b
		// number     b < a   b < a   b < a      (1)     a < b
		// string     b < a   b < a   b < a      b < a   (1)
		// EOA end of array
		// (0) continue on
		// (1) compare them via "<"

		// Handles first row in table
		if (i >= a.length) return i < b.length && (typeof b[i])[0] != "u";

		var aValue = a[i];
		var aType = (typeof aValue)[0];

		// Handles first column in table
		if (i >= b.length) return aType == "u";

		var bValue = b[i];
		var bType = (typeof bValue)[0];

		if (aType == bType) {
			if (aType != "o" && aType != "u" && aValue != bValue) {
				return aValue < bValue;
			}
			i++;
		} else {
			// Handles remaining cases
			if (aType == "o" && bType == "n") return true;
			return bType == "s" || aType == "u";
		}
	}
}
var rangeToString = function(range) {
	var fixCount = range[0];
	var str = "";
	if (range.length === 1) {
		return "*";
	} else if (fixCount + 0.5) {
		str +=
			fixCount == 0
				? ">="
				: fixCount == -1
				? "<"
				: fixCount == 1
				? "^"
				: fixCount == 2
				? "~"
				: fixCount > 0
				? "="
				: "!=";
		var needDot = 1;
		for (var i = 1; i < range.length; i++) {
			var item = range[i];
			var t = (typeof item)[0];
			needDot--;
			str +=
				t == "u"
					? // undefined: prerelease marker, add an "-"
					  "-"
					: // number or string: add the item, set flag to add an "." between two of them
					  (needDot > 0 ? "." : "") + ((needDot = 2), item);
		}
		return str;
	} else {
		var stack = [];
		for (var i = 1; i < range.length; i++) {
			var item = range[i];
			stack.push(
				item === 0
					? "not(" + pop() + ")"
					: item === 1
					? "(" + pop() + " || " + pop() + ")"
					: item === 2
					? stack.pop() + " " + stack.pop()
					: rangeToString(item)
			);
		}
		return pop();
	}
	function pop() {
		return stack.pop().replace(/^\((.+)\)$/, "$1");
	}
}
var satisfy = function(range, version) {
	if (0 in range) {
		version = parseVersion(version);
		var fixCount = /** @type {number} */ (range[0]);
		// when negated is set it swill set for < instead of >=
		var negated = fixCount < 0;
		if (negated) fixCount = -fixCount - 1;
		for (var i = 0, j = 1, isEqual = true; ; j++, i++) {
			// cspell:word nequal nequ

			// when isEqual = true:
			// range         version: EOA/object  undefined  number    string
			// EOA                    equal       block      big-ver   big-ver
			// undefined              bigger      next       big-ver   big-ver
			// number                 smaller     block      cmp       big-cmp
			// fixed number           smaller     block      cmp-fix   differ
			// string                 smaller     block      differ    cmp
			// fixed string           smaller     block      small-cmp cmp-fix

			// when isEqual = false:
			// range         version: EOA/object  undefined  number    string
			// EOA                    nequal      block      next-ver  next-ver
			// undefined              nequal      block      next-ver  next-ver
			// number                 nequal      block      next      next
			// fixed number           nequal      block      next      next   (this never happens)
			// string                 nequal      block      next      next
			// fixed string           nequal      block      next      next   (this never happens)

			// EOA end of array
			// equal (version is equal range):
			//   when !negated: return true,
			//   when negated: return false
			// bigger (version is bigger as range):
			//   when fixed: return false,
			//   when !negated: return true,
			//   when negated: return false,
			// smaller (version is smaller as range):
			//   when !negated: return false,
			//   when negated: return true
			// nequal (version is not equal range (> resp <)): return true
			// block (version is in different prerelease area): return false
			// differ (version is different from fixed range (string vs. number)): return false
			// next: continues to the next items
			// next-ver: when fixed: return false, continues to the next item only for the version, sets isEqual=false
			// big-ver: when fixed || negated: return false, continues to the next item only for the version, sets isEqual=false
			// next-nequ: continues to the next items, sets isEqual=false
			// cmp (negated === false): version < range => return false, version > range => next-nequ, else => next
			// cmp (negated === true): version > range => return false, version < range => next-nequ, else => next
			// cmp-fix: version == range => next, else => return false
			// big-cmp: when negated => return false, else => next-nequ
			// small-cmp: when negated => next-nequ, else => return false

			var rangeType = j < range.length ? (typeof range[j])[0] : "";

			var versionValue;
			var versionType;

			// Handles first column in both tables (end of version or object)
			if (
				i >= version.length ||
				((versionValue = version[i]),
				(versionType = (typeof versionValue)[0]) == "o")
			) {
				// Handles nequal
				if (!isEqual) return true;
				// Handles bigger
				if (rangeType == "u") return j > fixCount && !negated;
				// Handles equal and smaller: (range === EOA) XOR negated
				return (rangeType == "") != negated; // equal + smaller
			}

			// Handles second column in both tables (version = undefined)
			if (versionType == "u") {
				if (!isEqual || rangeType != "u") {
					return false;
				}
			}

			// switch between first and second table
			else if (isEqual) {
				// Handle diagonal
				if (rangeType == versionType) {
					if (j <= fixCount) {
						// Handles "cmp-fix" cases
						if (versionValue != range[j]) {
							return false;
						}
					} else {
						// Handles "cmp" cases
						if (negated ? versionValue > range[j] : versionValue < range[j]) {
							return false;
						}
						if (versionValue != range[j]) isEqual = false;
					}
				}

				// Handle big-ver
				else if (rangeType != "s" && rangeType != "n") {
					if (negated || j <= fixCount) return false;
					isEqual = false;
					j--;
				}

				// Handle differ, big-cmp and small-cmp
				else if (j <= fixCount || versionType < rangeType != negated) {
					return false;
				} else {
					isEqual = false;
				}
			} else {
				// Handles all "next-ver" cases in the second table
				if (rangeType != "s" && rangeType != "n") {
					isEqual = false;
					j--;
				}

				// next is applied by default
			}
		}
	}
	/** @type {(boolean | number)[]} */
	var stack = [];
	var p = stack.pop.bind(stack);
	for (var i = 1; i < range.length; i++) {
		var item = /** @type {SemVerRange | 0 | 1 | 2} */ (range[i]);
		stack.push(
			item == 1
				? p() | p()
				: item == 2
				? p() & p()
				: item
				? satisfy(item, version)
				: !p()
		);
	}
	return !!p();
}
var ensureExistence = function(scopeName, key) {
	var scope = __webpack_require__.S[scopeName];
	if(!scope || !__webpack_require__.o(scope, key)) throw new Error("Shared module " + key + " doesn't exist in shared scope " + scopeName);
	return scope;
};
var findVersion = function(scope, key) {
	var versions = scope[key];
	var key = Object.keys(versions).reduce(function(a, b) {
		return !a || versionLt(a, b) ? b : a;
	}, 0);
	return key && versions[key]
};
var findSingletonVersionKey = function(scope, key) {
	var versions = scope[key];
	return Object.keys(versions).reduce(function(a, b) {
		return !a || (!versions[a].loaded && versionLt(a, b)) ? b : a;
	}, 0);
};
var getInvalidSingletonVersionMessage = function(scope, key, version, requiredVersion) {
	return "Unsatisfied version " + version + " from " + (version && scope[key][version].from) + " of shared singleton module " + key + " (required " + rangeToString(requiredVersion) + ")"
};
var getSingleton = function(scope, scopeName, key, requiredVersion) {
	var version = findSingletonVersionKey(scope, key);
	return get(scope[key][version]);
};
var getSingletonVersion = function(scope, scopeName, key, requiredVersion) {
	var version = findSingletonVersionKey(scope, key);
	if (!satisfy(requiredVersion, version)) warn(getInvalidSingletonVersionMessage(scope, key, version, requiredVersion));
	return get(scope[key][version]);
};
var getStrictSingletonVersion = function(scope, scopeName, key, requiredVersion) {
	var version = findSingletonVersionKey(scope, key);
	if (!satisfy(requiredVersion, version)) throw new Error(getInvalidSingletonVersionMessage(scope, key, version, requiredVersion));
	return get(scope[key][version]);
};
var findValidVersion = function(scope, key, requiredVersion) {
	var versions = scope[key];
	var key = Object.keys(versions).reduce(function(a, b) {
		if (!satisfy(requiredVersion, b)) return a;
		return !a || versionLt(a, b) ? b : a;
	}, 0);
	return key && versions[key]
};
var getInvalidVersionMessage = function(scope, scopeName, key, requiredVersion) {
	var versions = scope[key];
	return "No satisfying version (" + rangeToString(requiredVersion) + ") of shared module " + key + " found in shared scope " + scopeName + ".\n" +
		"Available versions: " + Object.keys(versions).map(function(key) {
		return key + " from " + versions[key].from;
	}).join(", ");
};
var getValidVersion = function(scope, scopeName, key, requiredVersion) {
	var entry = findValidVersion(scope, key, requiredVersion);
	if(entry) return get(entry);
	throw new Error(getInvalidVersionMessage(scope, scopeName, key, requiredVersion));
};
var warn = function(msg) {
	if (typeof console !== "undefined" && console.warn) console.warn(msg);
};
var warnInvalidVersion = function(scope, scopeName, key, requiredVersion) {
	warn(getInvalidVersionMessage(scope, scopeName, key, requiredVersion));
};
var get = function(entry) {
	entry.loaded = 1;
	return entry.get()
};
var init = function(fn) { return function(scopeName, a, b, c) {
	var promise = __webpack_require__.I(scopeName);
	if (promise && promise.then) return promise.then(fn.bind(fn, scopeName, __webpack_require__.S[scopeName], a, b, c));
	return fn(scopeName, __webpack_require__.S[scopeName], a, b, c);
}; };

var load = /*#__PURE__*/ init(function(scopeName, scope, key) {
	ensureExistence(scopeName, key);
	return get(findVersion(scope, key));
});
var loadFallback = /*#__PURE__*/ init(function(scopeName, scope, key, fallback) {
	return scope && __webpack_require__.o(scope, key) ? get(findVersion(scope, key)) : fallback();
});
var loadVersionCheck = /*#__PURE__*/ init(function(scopeName, scope, key, version) {
	ensureExistence(scopeName, key);
	return get(findValidVersion(scope, key, version) || warnInvalidVersion(scope, scopeName, key, version) || findVersion(scope, key));
});
var loadSingleton = /*#__PURE__*/ init(function(scopeName, scope, key) {
	ensureExistence(scopeName, key);
	return getSingleton(scope, scopeName, key);
});
var loadSingletonVersionCheck = /*#__PURE__*/ init(function(scopeName, scope, key, version) {
	ensureExistence(scopeName, key);
	return getSingletonVersion(scope, scopeName, key, version);
});
var loadStrictVersionCheck = /*#__PURE__*/ init(function(scopeName, scope, key, version) {
	ensureExistence(scopeName, key);
	return getValidVersion(scope, scopeName, key, version);
});
var loadStrictSingletonVersionCheck = /*#__PURE__*/ init(function(scopeName, scope, key, version) {
	ensureExistence(scopeName, key);
	return getStrictSingletonVersion(scope, scopeName, key, version);
});
var loadVersionCheckFallback = /*#__PURE__*/ init(function(scopeName, scope, key, version, fallback) {
	if(!scope || !__webpack_require__.o(scope, key)) return fallback();
	return get(findValidVersion(scope, key, version) || warnInvalidVersion(scope, scopeName, key, version) || findVersion(scope, key));
});
var loadSingletonFallback = /*#__PURE__*/ init(function(scopeName, scope, key, fallback) {
	if(!scope || !__webpack_require__.o(scope, key)) return fallback();
	return getSingleton(scope, scopeName, key);
});
var loadSingletonVersionCheckFallback = /*#__PURE__*/ init(function(scopeName, scope, key, version, fallback) {
	if(!scope || !__webpack_require__.o(scope, key)) return fallback();
	return getSingletonVersion(scope, scopeName, key, version);
});
var loadStrictVersionCheckFallback = /*#__PURE__*/ init(function(scopeName, scope, key, version, fallback) {
	var entry = scope && __webpack_require__.o(scope, key) && findValidVersion(scope, key, version);
	return entry ? get(entry) : fallback();
});
var loadStrictSingletonVersionCheckFallback = /*#__PURE__*/ init(function(scopeName, scope, key, version, fallback) {
	if(!scope || !__webpack_require__.o(scope, key)) return fallback();
	return getStrictSingletonVersion(scope, scopeName, key, version);
});
var resolveHandler = function(data) {
	var strict = false
	var singleton = false
	var versionCheck = false
	var fallback = false
	var args = [data.shareScope, data.shareKey];
	if (data.requiredVersion) {
		if (data.strictVersion) strict = true;
		if (data.singleton) singleton = true;
		args.push(parseRange(data.requiredVersion));
		versionCheck = true
	} else if (data.singleton) singleton = true;
	if (data.fallback) {
		fallback = true;
		args.push(data.fallback);
	}
	if (strict && singleton && versionCheck && fallback) return function() { return loadStrictSingletonVersionCheckFallback.apply(null, args); }
	if (strict && versionCheck && fallback) return function() { return loadStrictVersionCheckFallback.apply(null, args); }
	if (singleton && versionCheck && fallback) return function() { return loadSingletonVersionCheckFallback.apply(null, args); }
	if (strict && singleton && versionCheck) return function() { return loadStrictSingletonVersionCheck.apply(null, args); }
	if (singleton && fallback) return function() { return loadSingletonFallback.apply(null, args); }
	if (versionCheck && fallback) return function() { return loadVersionCheckFallback.apply(null, args); }
	if (strict && versionCheck) return function() { return loadStrictVersionCheck.apply(null, args); }
	if (singleton && versionCheck) return function() { return loadSingletonVersionCheck.apply(null, args); }
	if (singleton) return function() { return loadSingleton.apply(null, args); }
	if (versionCheck) return function() { return loadVersionCheck.apply(null, args); }
	if (fallback) return function() { return loadFallback.apply(null, args); }
	return function() { return load.apply(null, args); }
};
var installedModules = {};
