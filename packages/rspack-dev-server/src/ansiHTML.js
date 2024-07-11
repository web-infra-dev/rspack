/* eslint-disable */
// @ts-nocheck
/**
 * The following code is modified based on
 * https://github.com/mahdyar/ansi-html-community/blob/b86cc3f1fa1d118477877352f0eafe1a70fd20ab/index.js
 *
 * Supported:
 *  - added support for 24-bit RGB colors.
 *
 * Apache 2.0 Licensed
 * Author @Tjatse
 * https://github.com/mahdyar/ansi-html-community/blob/master/LICENSE
 */
"use strict";

module.exports = ansiHTML;

// Reference to https://github.com/sindresorhus/ansi-regex
var _regANSI =
	/(?:(?:\u001b\[)|\u009b)(?:(?:[0-9]{1,3})?(?:(?:;[0-9]{0,3})*)?[A-M|f-m])|\u001b[A-M]/;

var _defColors = {
	reset: ["fff", "000"], // [FOREGROUND_COLOR, BACKGROUND_COLOR]
	black: "000",
	red: "ff0000",
	green: "209805",
	yellow: "e8bf03",
	blue: "0000ff",
	magenta: "ff00ff",
	cyan: "00ffee",
	lightgrey: "f0f0f0",
	darkgrey: "888"
};
var _styles = {
	30: "black",
	31: "red",
	32: "green",
	33: "yellow",
	34: "blue",
	35: "magenta",
	36: "cyan",
	37: "lightgrey"
};

var _colorMode = {
	2: "rgb"
};

var _openTags = {
	1: "font-weight:bold", // bold
	2: "opacity:0.5", // dim
	3: "<i>", // italic
	4: "<u>", // underscore
	8: "display:none", // hidden
	9: "<del>", // delete
	38: match => {
		// color
		var mode = _colorMode[match[0]];
		if (mode === "rgb") {
			var r, g, b;
			r = match[1];
			g = match[2];
			b = match[3];
			match.advance(4);
			return "color: rgb(" + r + "," + g + "," + b + ")";
		}
	},
	48: match => {
		// background color
		var mode = _colorMode[match[0]];
		if (mode === "rgb") {
			var r, g, b;
			r = match[1];
			g = match[2];
			b = match[3];
			match.advance(4);
			return "background-color: rgb(" + r + "," + g + "," + b + ")";
		}
	}
};

var _openTagToCloseTag = {
	3: "23",
	4: "24",
	9: "29"
};

var _closeTags = {
	0: ansiCodes => {
		if (!ansiCodes) return "</span>";
		if (!ansiCodes.length) return "";
		var code,
			ret = "";
		while ((code = ansiCodes.pop())) {
			var closeTag = _openTagToCloseTag[code];
			if (closeTag) {
				ret += _closeTags[closeTag];
				continue;
			}
			ret += "</span>";
		}
		return ret;
	},
	23: "</i>", // reset italic
	24: "</u>", // reset underscore
	29: "</del>" // reset delete
};

[21, 22, 27, 28, 39, 49].forEach(n => {
	_closeTags[n] = "</span>";
});

/**
 * Normalize ';<seq>' | '<seq>' -> '<seq>'
 * @param {string | null} seq
 * @returns {null | string}
 */
function normalizeSeq(seq) {
	if (seq === null || seq === undefined) return null;
	if (seq.startsWith(";")) {
		return seq.slice(1);
	}
	return seq;
}

/**
 * Converts text with ANSI color codes to HTML markup.
 * @param {String} text
 * @returns {*}
 */
function ansiHTML(text) {
	// Returns the text if the string has no ANSI escape code.
	if (!_regANSI.test(text)) {
		return text;
	}

	// Cache opened sequence.
	var ansiCodes = [];
	// Replace with markup.
	var ret = text.replace(/\033\[(?:[0-9]{1,3})?(?:(?:;[0-9]{0,3})*)?m/g, m => {
		var match = m.match(/(;?\d+)/g).map(normalizeSeq);
		Object.defineProperty(match, "advance", {
			value: function (count) {
				this.splice(0, count);
			}
		});
		var seq,
			rep = "";
		while ((seq = match[0])) {
			match.advance(1);
			rep += applySeq(seq);
		}
		return rep;

		function applySeq(seq) {
			var other = _openTags[seq];
			if (
				other &&
				(other = typeof other === "function" ? other(match) : other)
			) {
				// If reset signal is encountered, we have to reset everything.
				var ret = "";
				if (seq === "0") {
					ret += _closeTags[seq](ansiCodes);
				}
				// If current sequence has been opened, close it.
				if (!!~ansiCodes.indexOf(seq)) {
					// eslint-disable-line no-extra-boolean-cast
					ansiCodes.pop();
					return "</span>";
				}
				// Open tag.
				ansiCodes.push(seq);
				return (
					ret + (other[0] === "<" ? other : '<span style="' + other + ';">')
				);
			}

			var ct = _closeTags[seq];
			if (typeof ct === "function") {
				return ct(ansiCodes);
			} else if (ct) {
				// Pop sequence
				ansiCodes.pop();
				return ct;
			}
			return "";
		}
	});

	// Make sure tags are closed.
	var l = ansiCodes.length;
	l > 0 && (ret += Array(l + 1).join("</span>"));

	return ret;
}

/**
 * Customize colors.
 * @param {Object} colors reference to _defColors
 */
ansiHTML.setColors = colors => {
	if (typeof colors !== "object") {
		throw new Error("`colors` parameter must be an Object.");
	}

	var _finalColors = {};
	for (var key in _defColors) {
		var hex = colors.hasOwnProperty(key) ? colors[key] : null;
		if (!hex) {
			_finalColors[key] = _defColors[key];
			continue;
		}
		if ("reset" === key) {
			if (typeof hex === "string") {
				hex = [hex];
			}
			if (
				!Array.isArray(hex) ||
				hex.length === 0 ||
				hex.some(h => typeof h !== "string")
			) {
				throw new Error(
					"The value of `" +
						key +
						"` property must be an Array and each item could only be a hex string, e.g.: FF0000"
				);
			}
			var defHexColor = _defColors[key];
			if (!hex[0]) {
				hex[0] = defHexColor[0];
			}
			if (hex.length === 1 || !hex[1]) {
				hex = [hex[0]];
				hex.push(defHexColor[1]);
			}

			hex = hex.slice(0, 2);
		} else if (typeof hex !== "string") {
			throw new Error(
				"The value of `" + key + "` property must be a hex string, e.g.: FF0000"
			);
		}
		_finalColors[key] = hex;
	}
	_setTags(_finalColors);
};

/**
 * Reset colors.
 */
ansiHTML.reset = () => {
	_setTags(_defColors);
};

/**
 * Expose tags, including open and close.
 * @type {Object}
 */
ansiHTML.tags = {};

if (Object.defineProperty) {
	Object.defineProperty(ansiHTML.tags, "open", {
		get: () => _openTags
	});
	Object.defineProperty(ansiHTML.tags, "close", {
		get: () => _closeTags
	});
} else {
	ansiHTML.tags.open = _openTags;
	ansiHTML.tags.close = _closeTags;
}

function _setTags(colors) {
	// reset all
	_openTags["0"] =
		"font-weight:normal;opacity:1;color:#" +
		colors.reset[0] +
		";background:#" +
		colors.reset[1];
	// inverse
	_openTags["7"] =
		"color:#" + colors.reset[1] + ";background:#" + colors.reset[0];
	// dark grey
	_openTags["90"] = "color:#" + colors.darkgrey;

	for (var code in _styles) {
		var color = _styles[code];
		var oriColor = colors[color] || "000";
		_openTags[code] = "color:#" + oriColor;
		code = parseInt(code);
		_openTags[(code + 10).toString()] = "background:#" + oriColor;
	}
}

ansiHTML.reset();
