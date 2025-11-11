
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["__federation_expose_App"],
	{
		"./src/main.js": function (e, s, n) {
			(n.r(s), n.d(s, { default: () => o, loadApp: () => p }));
			var t = n("./src/App.svelte");
			const l = new t.A({
					target: document.querySelector("#app_04"),
					props: { name: "world" }
				}),
				p = e =>
					new t.A({
						target: document.querySelector("#app_04"),
						props: { name: "world" }
					});
			window.app = l;
			const o = l;
		},
		"./src/App.svelte": function (e, s, n) {
			n.d(s, { A: () => r });
			var t = n(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/index.js"
			);
			function l(e) {
				let s, n, l, p;
				return {
					c() {
						((s = (0, t.ND4)("h1")),
							(n = (0, t.Qq7)("Hello From Svelte ")),
							(l = (0, t.Qq7)(e[0])),
							(p = (0, t.Qq7)("!")),
							(0, t.CFu)(s, "class", "svelte-1ucbz36"));
					},
					m(e, o) {
						((0, t.Yry)(e, s, o),
							(0, t.BCw)(s, n),
							(0, t.BCw)(s, l),
							(0, t.BCw)(s, p));
					},
					p(e, [s]) {
						1 & s && (0, t.iQh)(l, e[0]);
					},
					i: t.lQ1,
					o: t.lQ1,
					d(e) {
						e && (0, t.YoD)(s);
					}
				};
			}
			function p(e, s, n) {
				let { name: t } = s;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && n(0, (t = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && n(0, (t = e.name));
					}),
					[t]
				);
			}
			(n(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/disclose-version/index.js"
			),
				n(
					"./src/App.svelte.0.css!=!../../../node_modules/.pnpm/svelte-loader@3.2.3_svelte@4.2.18/node_modules/svelte-loader/index.js?cssPath=/Users/zackjackson/rspack/examples/comprehensive-demo-react18/app-04/src/App.svelte.0.css!./src/App.svelte"
				));
			class o extends t.r7T {
				constructor(e) {
					(super(), (0, t.TsN)(this, e, p, l, t.jXN, { name: 0 }));
				}
			}
			const r = o;
		},
		"./src/App.svelte.0.css!=!../../../node_modules/.pnpm/svelte-loader@3.2.3_svelte@4.2.18/node_modules/svelte-loader/index.js?cssPath=/Users/zackjackson/rspack/examples/comprehensive-demo-react18/app-04/src/App.svelte.0.css!./src/App.svelte":
			function (e, s, n) {
				e.exports = {};
			}
	}
]);
