var version = typeof $RUNTIME_GET_VERSION$ === 'function' ? $RUNTIME_GET_VERSION$() : '';
var bundlerName = "$BUNDLER_NAME$";
__webpack_require__.ruid = "bundler=" + bundlerName + (version ? "@" + version : "");
