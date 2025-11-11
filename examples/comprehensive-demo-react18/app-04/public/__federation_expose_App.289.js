
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	[289],
	{
		89: (e, n, t) => {
			t.d(n, { A: () => l });
			var a = t(684);
			function o(e) {
				let n, t, o, r;
				return {
					c() {
						((n = (0, a.ND4)("h1")),
							(t = (0, a.Qq7)("Hello From Svelte ")),
							(o = (0, a.Qq7)(e[0])),
							(r = (0, a.Qq7)("!")),
							(0, a.CFu)(n, "class", "svelte-1ucbz36"));
					},
					m(e, s) {
						((0, a.Yry)(e, n, s),
							(0, a.BCw)(n, t),
							(0, a.BCw)(n, o),
							(0, a.BCw)(n, r));
					},
					p(e, [n]) {
						1 & n && (0, a.iQh)(o, e[0]);
					},
					i: a.lQ1,
					o: a.lQ1,
					d(e) {
						e && (0, a.YoD)(n);
					}
				};
			}
			function r(e, n, t) {
				let { name: a } = n;
				return (
					window.addEventListener("change-name", e => {
						(console.log(e), void 0 !== e.detail && t(0, (a = e.detail.name)));
					}),
					(e.$$set = e => {
						"name" in e && t(0, (a = e.name));
					}),
					[a]
				);
			}
			t(501);
			class s extends a.r7T {
				constructor(e) {
					(super(), (0, a.TsN)(this, e, r, o, a.jXN, { name: 0 }));
				}
			}
			const l = s;
		},
		803: (e, n, t) => {
			(t.r(n), t.d(n, { default: () => s, loadApp: () => r }));
			var a = t(89);
			const o = new a.A({
					target: document.querySelector("#app_04"),
					props: { name: "world" }
				}),
				r = e =>
					new a.A({
						target: document.querySelector("#app_04"),
						props: { name: "world" }
					});
			window.app = o;
			const s = o;
		}
	}
]);
