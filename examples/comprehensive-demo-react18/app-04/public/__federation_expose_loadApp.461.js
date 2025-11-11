
(self.webpackChunkapp4 = self.webpackChunkapp4 || []).push([
	[461],
	{
		89: (e, n, t) => {
			t.d(n, { A: () => o });
			var a = t(684);
			function s(e) {
				let n, t, s, r;
				return {
					c() {
						((n = (0, a.ND4)("h1")),
							(t = (0, a.Qq7)("Hello From Svelte ")),
							(s = (0, a.Qq7)(e[0])),
							(r = (0, a.Qq7)("!")),
							(0, a.CFu)(n, "class", "svelte-1ucbz36"));
					},
					m(e, l) {
						((0, a.Yry)(e, n, l),
							(0, a.BCw)(n, t),
							(0, a.BCw)(n, s),
							(0, a.BCw)(n, r));
					},
					p(e, [n]) {
						1 & n && (0, a.iQh)(s, e[0]);
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
			class l extends a.r7T {
				constructor(e) {
					(super(), (0, a.TsN)(this, e, r, s, a.jXN, { name: 0 }));
				}
			}
			const o = l;
		},
		995: (e, n, t) => {
			(t.r(n), t.d(n, { default: () => s }));
			var a = t(89);
			const s = (e, n) =>
				new a.A({
					target: document.querySelector(`#${e}`),
					props: { name: n }
				});
		}
	}
]);
