
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["__federation_expose_loadApp"],
	{
		"./src/App.svelte": (e, n, s) => {
			s.d(n, { A: () => a });
			var t = s(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/index.js"
			);
			function l(e) {
				let n, s, l, o;
				return {
					c() {
						((n = (0, t.ND4)("h1")),
							(s = (0, t.Qq7)("Hello From Svelte ")),
							(l = (0, t.Qq7)(e[0])),
							(o = (0, t.Qq7)("!")),
							(0, t.CFu)(n, "class", "svelte-1ucbz36"));
					},
					m(e, r) {
						((0, t.Yry)(e, n, r),
							(0, t.BCw)(n, s),
							(0, t.BCw)(n, l),
							(0, t.BCw)(n, o));
					},
					p(e, [n]) {
						1 & n && (0, t.iQh)(l, e[0]);
					},
					i: t.lQ1,
					o: t.lQ1,
					d(e) {
						e && (0, t.YoD)(n);
					}
				};
			}
			function o(e, n, s) {
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
			class r extends t.r7T {
				constructor(e) {
					(super(), (0, t.TsN)(this, e, o, l, t.jXN, { name: 0 }));
				}
			}
			const a = r;
		},
		"./src/loadApp.js": (e, n, s) => {
			(s.r(n), s.d(n, { default: () => l }));
			var t = s("./src/App.svelte");
			const l = (e, n) =>
				new t.A({
					target: document.querySelector(`#${e}`),
					props: { name: n }
				});
		}
	}
]);
