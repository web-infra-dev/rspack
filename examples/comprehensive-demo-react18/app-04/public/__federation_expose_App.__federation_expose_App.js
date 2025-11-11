
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["__federation_expose_App"],
	{
		"./src/App.svelte": (e, n, s) => {
			s.d(n, { A: () => p });
			var t = s(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/index.js"
			);
			function o(e) {
				let n, s, o, r;
				return {
					c() {
						((n = (0, t.ND4)("h1")),
							(s = (0, t.Qq7)("Hello From Svelte ")),
							(o = (0, t.Qq7)(e[0])),
							(r = (0, t.Qq7)("!")),
							(0, t.CFu)(n, "class", "svelte-1ucbz36"));
					},
					m(e, l) {
						((0, t.Yry)(e, n, l),
							(0, t.BCw)(n, s),
							(0, t.BCw)(n, o),
							(0, t.BCw)(n, r));
					},
					p(e, [n]) {
						1 & n && (0, t.iQh)(o, e[0]);
					},
					i: t.lQ1,
					o: t.lQ1,
					d(e) {
						e && (0, t.YoD)(n);
					}
				};
			}
			function r(e, n, s) {
				let { name: t } = n;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && s(0, (t = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && s(0, (t = e.name));
					}),
					[t]
				);
			}
			s(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/disclose-version/index.js"
			);
			class l extends t.r7T {
				constructor(e) {
					(super(), (0, t.TsN)(this, e, r, o, t.jXN, { name: 0 }));
				}
			}
			const p = l;
		},
		"./src/main.js": (e, n, s) => {
			(s.r(n), s.d(n, { default: () => l, loadApp: () => r }));
			var t = s("./src/App.svelte");
			const o = new t.A({
					target: document.querySelector("#app_04"),
					props: { name: "world" }
				}),
				r = e =>
					new t.A({
						target: document.querySelector("#app_04"),
						props: { name: "world" }
					});
			window.app = o;
			const l = o;
		}
	}
]);
