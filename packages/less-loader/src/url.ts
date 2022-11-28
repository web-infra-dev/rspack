import fs from "fs";
import path from "path";
import os from "os";
import qs from "qs";
export const resolvePathAndQuery = (originalPath: string) => {
	const [filePath, queryStr] = originalPath.split("?");
	const query: Record<string, any> = qs.parse(queryStr ?? "");

	for (const key of Object.keys(query)) {
		if (query[key] === "") {
			query[key] = true;
		}
	}

	return {
		query,
		rawQuery: queryStr,
		originalFilePath: filePath
	};
};

export const dataUrlRE = /^\s*data:/i;
export const isDataUrl = (url: string): boolean => dataUrlRE.test(url);
export const cssUrlRE =
	/(?<=^|[^\w\-\u0080-\uffff])url\(\s*('[^']+'|"[^"]+"|[^'")]+)\s*\)/;
type CssUrlReplacer = (
	url: string,
	importer?: string
) => string | Promise<string>;
export const externalRE = /^(https?:)?\/\//;
export const isWindows = os.platform() === "win32";
export function slash(p: string): string {
	return p.replace(/\\/g, "/");
}

export function normalizePath(id: string): string {
	return path.posix.normalize(isWindows ? slash(id) : id);
}
export const isExternalUrl = (url: string): boolean => externalRE.test(url);

/**
 * relative url() inside \@imported sass and less files must be rebased to use
 * root file as base.
 */
export async function rebaseUrls(
	file: string,
	rootDir: string,
	resolver: (id: string, dir: string) => string
): Promise<{ file: string; contents?: string }> {
	file = path.resolve(file); // ensure os-specific flashes
	// in the same dir, no need to rebase
	const fileDir = path.dirname(file);
	if (fileDir === rootDir) {
		return { file };
	}
	// no url()
	const content = fs.readFileSync(file, "utf-8");
	if (!cssUrlRE.test(content)) {
		return { file };
	}
	const rebased = await rewriteCssUrls(
		content,
		path.extname(file).slice(1),
		url => {
			if (url.startsWith("/")) return url;
			return resolver(url, fileDir);
		}
	);
	return {
		file,
		contents: rebased
	};
}

export function rewriteCssUrls(
	css: string,
	type: false | string,
	replacer: CssUrlReplacer
): Promise<string> {
	return asyncReplace(css, cssUrlRE, async match => {
		const [matched, rawUrl] = match;
		if (
			(type === "less" && rawUrl.startsWith("@")) ||
			((type === "sass" || type === "scss") && rawUrl.startsWith("$"))
		) {
			return `url(${rawUrl})`;
		}
		return await doUrlReplace(rawUrl, matched, replacer);
	});
}

export async function asyncReplace(
	input: string,
	re: RegExp,
	replacer: (match: RegExpExecArray) => string | Promise<string>
): Promise<string> {
	let match: RegExpExecArray | null;
	let remaining = input;
	let rewritten = "";
	while ((match = re.exec(remaining))) {
		rewritten += remaining.slice(0, match.index);
		rewritten += await replacer(match);
		remaining = remaining.slice(match.index + match[0].length);
	}
	rewritten += remaining;
	return rewritten;
}

async function doUrlReplace(
	rawUrl: string,
	matched: string,
	replacer: CssUrlReplacer
) {
	let wrap = "";
	const first = rawUrl[0];
	if (first === `"` || first === `'`) {
		wrap = first;
		rawUrl = rawUrl.slice(1, -1);
	}
	if (isExternalUrl(rawUrl) || isDataUrl(rawUrl) || rawUrl.startsWith("#")) {
		return matched;
	}

	return `url(${wrap}${await replacer(rawUrl)}${wrap})`;
}

export const getFilePathWithPlatform = (
	originalFilePath: string,
	platform: string
): string => {
	let filePath = originalFilePath;
	if (platform) {
		filePath = filePath.replace(
			new RegExp(`${path.extname(filePath)}$`, "g"),
			`.${platform}${path.extname(filePath)}`
		);
		return fs.existsSync(filePath) ? filePath : originalFilePath;
	}
	return originalFilePath;
};
