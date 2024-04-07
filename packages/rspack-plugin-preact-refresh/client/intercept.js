/**
 * The following code is modified based on
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/packages/webpack/src/utils/Runtime.js
 *
 * MIT Licensed
 * Author JoviDeCroock
 * Copyright (c) 2021-Present Preact Team
 * https://github.com/preactjs/prefresh/blob/018f5cc907629b82ffb201c32e948efe4b40098a/LICENSE
 */

__webpack_require__.i.push(function (options) {
	var originalFactory = options.factory;
	options.factory = function (moduleObject, moduleExports, webpackRequire) {
		var prevRefreshReg = self.$RefreshReg$;
		var prevRefreshSig = self.$RefreshSig$;
		self.$RefreshSig$ = function () {
			var status = "begin";
			var savedType;

			return function (type, key, forceReset, getCustomHooks) {
				if (!savedType) savedType = type;
				status = self.__PREFRESH__.sign(
					type || savedType,
					key,
					forceReset,
					getCustomHooks,
					status
				);
				return type;
			};
		};
		var reg = function (currentModuleId) {
			self.$RefreshReg$ = function (type, id) {
				self.__PREFRESH__.register(type, currentModuleId + " " + id);
			};
		};
		reg();
		try {
			originalFactory.call(this, moduleObject, moduleExports, webpackRequire);
		} finally {
			self.$RefreshReg$ = prevRefreshReg;
			self.$RefreshSig$ = prevRefreshSig;
		}
	};
});
