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
interface AnsiHtmlTags {
	open: typeof _openTags;
	close: typeof _closeTags;
}

type Option<T> = T | null | undefined;

type Match = {
	advance: (n: number) => void;
} & Array<string>;

// Reference to https://github.com/sindresorhus/ansi-regex
const _regANSI =
	/(?:(?:\u001b\[)|\u009b)(?:(?:[0-9]{1,3})?(?:(?:;[0-9]{0,3})*)?[A-M|f-m])|\u001b[A-M]/;

const _defColors: Record<string, string | Array<string>> = {
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
const _styles: Record<string, string> = {
	30: "black",
	31: "red",
	32: "green",
	33: "yellow",
	34: "blue",
	35: "magenta",
	36: "cyan",
	37: "lightgrey"
};

const _colorMode: Record<string, string> = {
	2: "rgb"
};

const _openTags: Record<string, string | ((m: Match) => Option<string>)> = {
	1: "font-weight:bold", // bold
	2: "opacity:0.5", // dim
	3: "<i>", // italic
	4: "<u>", // underscore
	8: "display:none", // hidden
	9: "<del>", // delete
	38: (match: Match) => {
		// color
		const mode = _colorMode[match[0]];
		if (mode === "rgb") {
			const r = match[1];
			const g = match[2];
			const b = match[3];
			match.advance(4);
			return `color: rgb(${r},${g},${b})`;
		}
	},
	48: (match: Match) => {
		// background color
		const mode = _colorMode[match[0]];
		if (mode === "rgb") {
			const r = match[1];
			const g = match[2];
			const b = match[3];
			match.advance(4);
			return `background-color: rgb(${r},${g},${b})`;
		}
	}
};

const _openTagToCloseTag: Record<string, string> = {
	3: "23",
	4: "24",
	9: "29"
};

const _closeTags: Record<
	string,
	string | ((ansiCodes: Option<Array<string>>) => string)
> = {
	0: ansiCodes => {
		if (!ansiCodes) return "</span>";
		if (!ansiCodes.length) return "";
		let code: Option<string>;
		let ret = "";
		while ((code = ansiCodes.pop())) {
			const closeTag = _openTagToCloseTag[code];
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

for (const n of [21, 22, 27, 28, 39, 49]) {
	_closeTags[n] = "</span>";
}

/**
 * Normalize ';<seq>' | '<seq>' -> '<seq>'
 */
function normalizeSeq(seq: Option<string>): Option<string> {
	if (seq === null || seq === undefined) return null;
	if (seq.startsWith(";")) {
		return seq.slice(1);
	}
	return seq;
}

/**
 * Converts text with ANSI color codes to HTML markup.
 */
export default function ansiHTML(text: string) {
	// Returns the text if the string has no ANSI escape code.
	if (!_regANSI.test(text)) {
		return text;
	}

	// Cache opened sequence.
	const ansiCodes: string[] = [];
	// Replace with markup.
	//@ts-ignore TS1487 error
	let ret = text.replace(/\033\[(?:[0-9]{1,3})?(?:(?:;[0-9]{0,3})*)?m/g, m => {
		const match = m.match(/(;?\d+)/g)?.map(normalizeSeq) as unknown as Match;
		Object.defineProperty(match, "advance", {
			value: function (count: number) {
				this.splice(0, count);
			}
		});
		let rep = "";
		let seq: string;
		while ((seq = match[0])) {
			match.advance(1);
			rep += applySeq(seq);
		}
		return rep;

		function applySeq(seq: string) {
			let other = _openTags[seq];
			if (
				other &&
				(other = typeof other === "function" ? (other(match) as string) : other)
			) {
				// If reset signal is encountered, we have to reset everything.
				let ret = "";
				if (seq === "0") {
					ret += (
						_closeTags[seq] as (ansiCodes: Option<Array<string>>) => string
					)(ansiCodes);
				}
				// If current sequence has been opened, close it.
				if (ansiCodes.indexOf(seq) !== -1) {
					ansiCodes.pop();
					return "</span>";
				}
				// Open tag.
				ansiCodes.push(seq);
				return ret + (other[0] === "<" ? other : `<span style="${other};">`);
			}

			const ct = _closeTags[seq];
			if (typeof ct === "function") {
				return ct(ansiCodes);
			}
			if (ct) {
				// Pop sequence
				ansiCodes.pop();
				return ct;
			}
			return "";
		}
	});

	// Make sure tags are closed.
	const l = ansiCodes.length;
	l > 0 && (ret += Array(l + 1).join("</span>"));

	return ret;
}

/**
 * Customize colors.
 * @param {Object} colors reference to _defColors
 */
ansiHTML.setColors = (colors: typeof _defColors) => {
	if (typeof colors !== "object") {
		throw new Error("`colors` parameter must be an Object.");
	}

	const _finalColors: typeof _defColors = {};
	for (const key in _defColors) {
		let hex = colors.hasOwnProperty(key) ? colors[key] : null;
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
					`The value of \`${key}\` property must be an Array and each item could only be a hex string, e.g.: FF0000`
				);
			}
			const defHexColor = _defColors[key];
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
				`The value of \`${key}\` property must be a hex string, e.g.: FF0000`
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
ansiHTML.tags = {} as AnsiHtmlTags;

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

function _setTags(colors: typeof _defColors) {
	// reset all
	_openTags["0"] =
		`font-weight:normal;opacity:1;color:#${colors.reset[0]};background:#${colors.reset[1]}`;
	// inverse
	_openTags["7"] = `color:#${colors.reset[1]};background:#${colors.reset[0]}`;
	// dark grey
	_openTags["90"] = `color:#${colors.darkgrey}`;

	for (const code in _styles) {
		const color = _styles[code];
		const oriColor = colors[color] || "000";
		_openTags[code] = `color:#${oriColor}`;
		const codeInt = Number.parseInt(code);
		_openTags[(codeInt + 10).toString()] = `background:#${oriColor}`;
	}
}

ansiHTML.reset();
