// Simulate what happens to lodash chunk after macro transformation
var __webpack_modules__ = {
	// Used lodash modules (should remain)
	map: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		var _baseMap = __webpack_require__("_baseMap");
		module.exports = function (collection, iteratee) {
			return _baseMap(collection, iteratee);
		};
	},

	_baseMap: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		module.exports = function (collection, iteratee) {
			/* implementation */
		};
	},

	filter: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		var _baseFilter = __webpack_require__("_baseFilter");
		module.exports = function (collection, predicate) {
			return _baseFilter(collection, predicate);
		};
	},

	_baseFilter: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		module.exports = function (collection, predicate) {
			/* implementation */
		};
	},

	// Unused lodash modules (after macro removed the conditional import)
	// These should become unreachable after the entry point no longer calls them
	add: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		var _baseAdd = __webpack_require__("_baseAdd");
		module.exports = function (a, b) {
			return _baseAdd(a, b);
		};
	},

	_baseAdd: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		module.exports = function (a, b) {
			return a + b;
		};
	},

	subtract: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		var _baseSubtract = __webpack_require__("_baseSubtract");
		module.exports = function (a, b) {
			return _baseSubtract(a, b);
		};
	},

	_baseSubtract: function (
		__unused_webpack_module,
		__webpack_exports__,
		__webpack_require__
	) {
		module.exports = function (a, b) {
			return a - b;
		};
	}
};

// Entry points (after macro transformation removed unused imports)
// Only map and filter are called - add and subtract should become unreachable
__webpack_require__("map");
__webpack_require__("filter");
// The macro removed these lines:
// __webpack_require__("add");      // <- macro removed this
// __webpack_require__("subtract"); // <- macro removed this
