const path = require("path");
const fs = require("fs");
const prettier = require("prettier");

const WORKSPACE_ROOT = path.resolve(__dirname, "../../");

const RSPACK_TEST = "packages/rspack/tests";
const WEBPACK_TEST = "webpack-test";

async function getCommon(
	baseA,
	baseB,
	result = new Set(),
	difference = new Set()
) {
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

	set = Array.from(set).map(async item => {
		let old = path.join(WORKSPACE_ROOT, RSPACK_TEST);
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
				await trimAndCompare();
			}

			async function trimAndCompare() {
				let aa = a.toString().trim();
				let bb = b.toString().trim();
				try {
					// Assume they share the same file type
					const option = {
						filepath: nextA
					};
					aa = await prettier.format(aa, option);
					bb = await prettier.format(bb, option);
				} catch (e) {}

				if (aa !== bb) {
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
			await getCommon(nextA, nextB, result, difference);
		}
	});

	await Promise.all(set);
}

/**
 * The following code is copied from
 * https://github.com/sindresorhus/binary-extensions/blob/40e44b510d87a63dcf42300bc8fbcb105f45a61c/binary-extensions.json
 *
 * MIT Licensed
 * Author Sindre Sorhus @sindresorhus
 */

const BINARY_EXT = [
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

function isBinaryPath(p) {
	return BINARY_EXT.includes(path.extname(p).slice(1));
}

let identical = new Set();
let difference = new Set();
getCommon(
	path.join(WORKSPACE_ROOT, RSPACK_TEST),
	path.join(WORKSPACE_ROOT, WEBPACK_TEST),
	identical,
	difference
).then(() => {
	if (difference.size > 0) {
		const excludeList = require("./diff-exclude.cjs").map(item => {
			if (item instanceof RegExp) {
				return i => item.test(i);
			}
			if (typeof item === "string") {
				return i => i.startsWith(item);
			}
			throw new Error(
				"exclude item should only be the type of `RegExp` or `string`"
			);
		});
		let retained = Array.from(difference).filter(
			item => !excludeList.some(fn => fn(item))
		);
		if (retained.length > 0) {
			console.log(
				"Due to the historical fact that rspack mixed webpack tests and rspack tests together, so this test is served as a helper to decouple these tests." +
					"\n\n" +
					"The following cases share the same name with webpack, however their content are not identical." +
					"\n" +
					"This would cause misunderstandings between those tests. This file can be removed after the old tests are no longer coupled with webpack tests." +
					"\n\n" +
					"Either ignore these files in the `" +
					path.join(WORKSPACE_ROOT, "scripts/test/diff-exclude.cjs") +
					"` with reason (MUST BE CAUTIOUS) or align it with webpack.\n"
			);
			console.log(new Set(retained.sort()));
			process.exit(1);
		}
	}
});
