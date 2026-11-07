import cases from './cases.mjs';

const imports = {
	"catalog": () => import("catalog"),
	"catalog-singleton": () => import("catalog-singleton"),
	"workspace": () => import("workspace"),
	"workspace-range": () => import("workspace-range"),
	"file": () => import("file"),
	"tag": () => import("tag"),
	"empty": () => import("empty"),
	"url-without-version": () => import("url-without-version"),
	"optional-priority": () => import("optional-priority"),
	"semver": () => import("semver"),
	"git-short": () => import("git-short"),
	"git-extreme-short": () => import("git-extreme-short"),
	"git-ssh": () => import("git-ssh"),
	"git-https": () => import("git-https"),
	"git-tree": () => import("git-tree"),
	"git-encoded": () => import("git-encoded"),
	"explicit-override": () => import("explicit-override"),
	"explicit-false": () => import("explicit-false"),
	"semver-mismatch": () => import("semver-mismatch"),
	"git-mismatch": () => import("git-mismatch"),
	"gitlab": () => import("gitlab"),
	"bitbucket": () => import("bitbucket"),
	"gist": () => import("gist"),
	"invalid-url": () => import("invalid-url"),
	"github-blob": () => import("github-blob"),
	"gist-raw": () => import("gist-raw")
};

if (!__webpack_require__.federation) {
	__webpack_share_scopes__.default = Object.fromEntries(
		cases.map(({ name, version }) => [name, {
			[version]: { get: () => () => name, from: "test-provider" }
		}])
	);
}

for (const { name, reject } of cases) {
	it(`should preserve sharing behavior for ${name}`, async () => {
		const oldWarn = console.warn;
		const warnings = [];
		console.warn = message => warnings.push(message);
		try {
			if (reject) {
				await expect(imports[name]()).rejects.toThrow();
			} else {
				const result = await imports[name]();
				expect(result.default).toBe(name);
				expect(warnings).toEqual([]);
			}
		} finally {
			console.warn = oldWarn;
		}
	});
}
