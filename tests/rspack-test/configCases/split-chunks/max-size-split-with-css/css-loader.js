const cssLoader = require('css-loader')

module.exports = function cssProxyLoader(code) {
	cssLoader.call(this, code)
}
