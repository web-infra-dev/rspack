
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	["__federation_expose_loadApp"],
	{
		"./src/loadApp.js": function (e, s, l) {
			(l.r(s), l.d(s, { default: () => t }));
			var n = l("./src/App.svelte");
			const t = (e, s) =>
				new n.A({
					target: document.querySelector(`#${e}`),
					props: { name: s }
				});
		},
		"./src/App.svelte": function (e, s, l) {
			l.d(s, { A: () => r });
			var n = l(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/index.js"
			);
			function t(e) {
				let s, l, t, o;
				return {
					c() {
						((s = (0, n.ND4)("h1")),
							(l = (0, n.Qq7)("Hello From Svelte ")),
							(t = (0, n.Qq7)(e[0])),
							(o = (0, n.Qq7)("!")),
							(0, n.CFu)(s, "class", "svelte-1ucbz36"));
					},
					m(e, p) {
						((0, n.Yry)(e, s, p),
							(0, n.BCw)(s, l),
							(0, n.BCw)(s, t),
							(0, n.BCw)(s, o));
					},
					p(e, [s]) {
						1 & s && (0, n.iQh)(t, e[0]);
					},
					i: n.lQ1,
					o: n.lQ1,
					d(e) {
						e && (0, n.YoD)(s);
					}
				};
			}
			function o(e, s, l) {
				let { name: n } = s;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && l(0, (n = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && l(0, (n = e.name));
					}),
					[n]
				);
			}
			(l(
				"../../../node_modules/.pnpm/svelte@4.2.18/node_modules/svelte/src/runtime/internal/disclose-version/index.js"
			),
				l(
					"./src/App.svelte.0.css!=!../../../node_modules/.pnpm/svelte-loader@3.2.3_svelte@4.2.18/node_modules/svelte-loader/index.js?cssPath=/Users/zackjackson/rspack/examples/comprehensive-demo-react18/app-04/src/App.svelte.0.css!./src/App.svelte"
				));
			class p extends n.r7T {
				constructor(e) {
					(super(), (0, n.TsN)(this, e, o, t, n.jXN, { name: 0 }));
				}
			}
			const r = p;
		},
		"./src/App.svelte.0.css!=!../../../node_modules/.pnpm/svelte-loader@3.2.3_svelte@4.2.18/node_modules/svelte-loader/index.js?cssPath=/Users/zackjackson/rspack/examples/comprehensive-demo-react18/app-04/src/App.svelte.0.css!./src/App.svelte":
			function (e, s, l) {
				e.exports = {};
			}
	}
]);
