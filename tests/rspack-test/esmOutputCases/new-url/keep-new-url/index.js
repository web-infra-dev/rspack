
function neverUse() {
	new URL('./asset.js', import.meta.url)(function foo() {})();
}
neverUse.bind(neverUse)
