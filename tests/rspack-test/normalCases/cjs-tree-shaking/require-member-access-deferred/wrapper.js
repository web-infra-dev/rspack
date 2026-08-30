exports.setup = (obj, constructorArgs) => {
	obj._impl = new Impl.implementation(constructorArgs);
	return obj;
};

class URL {
	constructor(url) {
		return exports.setup(Object.create(URL.prototype), [url]);
	}
}

exports.interface = URL;

const Impl = require("./impl");
