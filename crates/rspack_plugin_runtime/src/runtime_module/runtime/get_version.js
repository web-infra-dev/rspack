var version = "$VERSION$";
if (typeof $RUNTIME_EXPOSE_GLOBAL$ === "object") {
	$RUNTIME_EXPOSE_GLOBAL$.version = version;
}
__webpack_require__.rv = function () {
	return version;
};
