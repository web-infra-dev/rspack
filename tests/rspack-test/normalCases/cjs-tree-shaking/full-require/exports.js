module.exports = function () {
	return "abc";
};

module.exports.a = "abc";

module.exports.b = function () {
	return "abc";
};

module.exports.c = {
	d: function () {
		return "abc";
	}
};

module.exports.e = {
	f: function () {
		return {
			g: {
				h: "abc"
			}
		};
	}
};

module.exports.h = {
	i: function () {
		return {
			j: {
				k: function () {
					return "abc";
				}
			}
		};
	}
};

module.exports.l = {
	m: function () {
		return function () {
			return {
				n: "abc"
			};
		};
	}
};
