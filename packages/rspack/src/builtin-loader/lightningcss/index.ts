// code modified based on https://github.com/parcel-bundler/lightningcss/blob/34b67a431c043fda5d4979bcdccb3008d082e243/node/browserslistToTargets.js

/**
MIT License

Copyright (c) 2021-present Devon Govett

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

const BROWSER_MAPPING: Record<string, string | null> = {
	and_chr: "chrome",
	and_ff: "firefox",
	ie_mob: "ie",
	op_mob: "opera",
	and_qq: null,
	and_uc: null,
	baidu: null,
	bb: null,
	kaios: null,
	op_mini: null
};

export function browserslistToTargets(
	browserslist: string[]
): Record<string, number> {
	const targets: Record<string, number> = {};
	for (const browser of browserslist) {
		const [name, v] = browser.split(" ");
		if (BROWSER_MAPPING[name] === null) {
			continue;
		}

		const version = parseVersion(v);
		if (version == null) {
			continue;
		}

		if (targets[name] == null || version < targets[name]) {
			targets[name] = version;
		}
	}

	return targets;
}

function parseVersion(version: string) {
	const [major, minor = 0, patch = 0] = version
		.split("-")[0]
		.split(".")
		.map(v => Number.parseInt(v, 10));

	if (isNaN(major) || isNaN(minor) || isNaN(patch)) {
		return null;
	}

	return (major << 16) | (minor << 8) | patch;
}

export enum Features {
	Empty = /*													*/ 0,
	Nesting = /*												*/ 1 << 0,
	NotSelectorList = /*								*/ 1 << 1,
	DirSelector = /*										*/ 1 << 2,
	LangSelectorList = /*								*/ 1 << 3,
	IsSelector = /*											*/ 1 << 4,
	TextDecorationThicknessPercent = /*	*/ 1 << 5,
	MediaIntervalSyntax = /*						*/ 1 << 6,
	MediaRangeSyntax = /*								*/ 1 << 7,
	CustomMediaQueries = /*							*/ 1 << 8,
	ClampFunction = /*									*/ 1 << 9,
	ColorFunction = /*									*/ 1 << 10,
	OklabColors = /*										*/ 1 << 11,
	LabColors = /* 											*/ 1 << 12,
	P3Colors = /*												*/ 1 << 13,
	HexAlphaColors = /*									*/ 1 << 14,
	SpaceSeparatedColorNotation = /*	 	*/ 1 << 15,
	FontFamilySystemUi = /*							*/ 1 << 16,
	DoublePositionGradients = /*				*/ 1 << 17,
	VendorPrefixes = /*									*/ 1 << 18,
	LogicalProperties = /*							*/ 1 << 19,
	Selectors = Features.Nesting |
		Features.NotSelectorList |
		Features.DirSelector |
		Features.LangSelectorList |
		Features.IsSelector,
	MediaQueries = Features.MediaIntervalSyntax |
		Features.MediaRangeSyntax |
		Features.CustomMediaQueries,
	Color = Features.ColorFunction |
		Features.OklabColors |
		Features.LabColors |
		Features.P3Colors |
		Features.HexAlphaColors |
		Features.SpaceSeparatedColorNotation
}

export interface Targets {
	android?: number;
	chrome?: number;
	edge?: number;
	firefox?: number;
	ie?: number;
	ios_saf?: number;
	opera?: number;
	safari?: number;
	samsung?: number;
}

export interface Drafts {
	/** Whether to enable @custom-media rules. */
	customMedia?: boolean;
}

export interface NonStandard {
	/** Whether to enable the non-standard >>> and /deep/ selector combinators used by Angular and Vue. */
	deepSelectorCombinator?: boolean;
}

export interface PseudoClasses {
	hover?: string;
	active?: string;
	focus?: string;
	focusVisible?: string;
	focusWithin?: string;
}

export type LoaderOptions = {
	errorRecovery?: boolean;
	targets?: Targets | string[] | string;
	include?: Features;
	exclude?: Features;
	draft?: Drafts;
	nonStandard?: NonStandard;
	pseudoClasses?: PseudoClasses;
	unusedSymbols?: Set<String>;
};
