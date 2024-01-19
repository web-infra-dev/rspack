const path = require("path");
const fs = require("fs");

const RSPACK_TEST = "packages/rspack/tests";
const WEBPACK_TEST = "webpack-test";

function getCommon(baseA, baseB, result = new Set(), difference = new Set()) {
	const a = fs.readdirSync(baseA, {
		withFileTypes: true
	});
	const b = fs.readdirSync(baseB, {
		withFileTypes: true
	});

	let set = new Set();
	let ap = a.map(item => item.name);
	let bp = b.map(item => item.name);
	for (let i = 0; i < a.length; i++) {
		if (bp.includes(a[i].name)) {
			set.add(a[i].name);
		}
	}
	for (let i = 0; i < b.length; i++) {
		if (ap.includes(b[i].name)) {
			set.add(b[i].name);
		}
	}

	set.forEach(item => {
		let old = path.join(__dirname, RSPACK_TEST);
		let nextA = path.join(baseA, item);
		let nextB = path.join(baseB, item);
		if (fs.lstatSync(nextA).isFile() && fs.lstatSync(nextB).isFile()) {
			let p = path.relative(old, nextA);
			let withoutEnd = p.split(path.sep).slice(0, -1).join(path.sep);
			let a = fs.readFileSync(nextA);
			let b = fs.readFileSync(nextB);

			if (isBinaryPath(p)) {
				bufferCompare();
			} else {
				trimAndCompare();
			}

			function trimAndCompare() {
				let s = a.toString().trim();
				let ss = b.toString().trim();
				if (s !== ss) {
					difference.add(p);
				} else {
					result.add(p);
				}
			}

			function bufferCompare() {
				if (!a.equals(b)) {
					difference.add(p);
				} else {
					result.add(p);
				}
			}
		} else {
			getCommon(nextA, nextB, result, difference);
		}
	});
}

function isBinaryPath(p) {
	const ext = [
		"3dm",
		"3ds",
		"3g2",
		"3gp",
		"7z",
		"a",
		"aac",
		"adp",
		"ai",
		"aif",
		"aiff",
		"alz",
		"ape",
		"apk",
		"appimage",
		"ar",
		"arj",
		"asf",
		"au",
		"avi",
		"bak",
		"baml",
		"bh",
		"bin",
		"bk",
		"bmp",
		"btif",
		"bz2",
		"bzip2",
		"cab",
		"caf",
		"cgm",
		"class",
		"cmx",
		"cpio",
		"cr2",
		"cur",
		"dat",
		"dcm",
		"deb",
		"dex",
		"djvu",
		"dll",
		"dmg",
		"dng",
		"doc",
		"docm",
		"docx",
		"dot",
		"dotm",
		"dra",
		"DS_Store",
		"dsk",
		"dts",
		"dtshd",
		"dvb",
		"dwg",
		"dxf",
		"ecelp4800",
		"ecelp7470",
		"ecelp9600",
		"egg",
		"eol",
		"eot",
		"epub",
		"exe",
		"f4v",
		"fbs",
		"fh",
		"fla",
		"flac",
		"flatpak",
		"fli",
		"flv",
		"fpx",
		"fst",
		"fvt",
		"g3",
		"gh",
		"gif",
		"graffle",
		"gz",
		"gzip",
		"h261",
		"h263",
		"h264",
		"icns",
		"ico",
		"ief",
		"img",
		"ipa",
		"iso",
		"jar",
		"jpeg",
		"jpg",
		"jpgv",
		"jpm",
		"jxr",
		"key",
		"ktx",
		"lha",
		"lib",
		"lvp",
		"lz",
		"lzh",
		"lzma",
		"lzo",
		"m3u",
		"m4a",
		"m4v",
		"mar",
		"mdi",
		"mht",
		"mid",
		"midi",
		"mj2",
		"mka",
		"mkv",
		"mmr",
		"mng",
		"mobi",
		"mov",
		"movie",
		"mp3",
		"mp4",
		"mp4a",
		"mpeg",
		"mpg",
		"mpga",
		"mxu",
		"nef",
		"npx",
		"numbers",
		"nupkg",
		"o",
		"odp",
		"ods",
		"odt",
		"oga",
		"ogg",
		"ogv",
		"otf",
		"ott",
		"pages",
		"pbm",
		"pcx",
		"pdb",
		"pdf",
		"pea",
		"pgm",
		"pic",
		"png",
		"pnm",
		"pot",
		"potm",
		"potx",
		"ppa",
		"ppam",
		"ppm",
		"pps",
		"ppsm",
		"ppsx",
		"ppt",
		"pptm",
		"pptx",
		"psd",
		"pya",
		"pyc",
		"pyo",
		"pyv",
		"qt",
		"rar",
		"ras",
		"raw",
		"resources",
		"rgb",
		"rip",
		"rlc",
		"rmf",
		"rmvb",
		"rpm",
		"rtf",
		"rz",
		"s3m",
		"s7z",
		"scpt",
		"sgi",
		"shar",
		"snap",
		"sil",
		"sketch",
		"slk",
		"smv",
		"snk",
		"so",
		"stl",
		"suo",
		"sub",
		"swf",
		"tar",
		"tbz",
		"tbz2",
		"tga",
		"tgz",
		"thmx",
		"tif",
		"tiff",
		"tlz",
		"ttc",
		"ttf",
		"txz",
		"udf",
		"uvh",
		"uvi",
		"uvm",
		"uvp",
		"uvs",
		"uvu",
		"viv",
		"vob",
		"war",
		"wav",
		"wax",
		"wbmp",
		"wdp",
		"weba",
		"webm",
		"webp",
		"whl",
		"wim",
		"wm",
		"wma",
		"wmv",
		"wmx",
		"woff",
		"woff2",
		"wrm",
		"wvx",
		"xbm",
		"xif",
		"xla",
		"xlam",
		"xls",
		"xlsb",
		"xlsm",
		"xlsx",
		"xlt",
		"xltm",
		"xltx",
		"xm",
		"xmind",
		"xpi",
		"xpm",
		"xwd",
		"xz",
		"z",
		"zip",
		"zipx"
	];
	return ext.includes(path.extname(p).slice(1));
}

let set = new Set();
let set2 = new Set();
getCommon(
	path.join(__dirname, RSPACK_TEST),
	path.join(__dirname, WEBPACK_TEST),
	set,
	set2
);
console.log(set2);
