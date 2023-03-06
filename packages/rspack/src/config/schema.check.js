/** This file was automatically generated, Run `pnpm precompile-schema` to update */
"use strict";
(module.exports = V), (module.exports.default = V);
const e = {
		cache: { $ref: "#/definitions/CacheOptions" },
		context: { $ref: "#/definitions/Context" },
		dependencies: { $ref: "#/definitions/Dependencies" },
		devServer: { $ref: "#/definitions/DevServer" },
		devtool: { $ref: "#/definitions/DevTool" },
		entry: { $ref: "#/definitions/Entry" },
		experiments: { $ref: "#/definitions/Experiments" },
		externals: { $ref: "#/definitions/Externals" },
		externalsType: { $ref: "#/definitions/ExternalsType" },
		infrastructureLogging: { $ref: "#/definitions/InfrastructureLogging" },
		mode: { $ref: "#/definitions/Mode" },
		module: { $ref: "#/definitions/ModuleOptions" },
		name: { $ref: "#/definitions/Name" },
		node: { $ref: "#/definitions/Node" },
		optimization: { $ref: "#/definitions/Optimization" },
		output: { $ref: "#/definitions/Output" },
		plugins: { $ref: "#/definitions/Plugins" },
		resolve: { $ref: "#/definitions/Resolve" },
		snapshot: { $ref: "#/definitions/SnapshotOptions" },
		stats: { $ref: "#/definitions/StatsValue" },
		target: { $ref: "#/definitions/Target" },
		watch: { $ref: "#/definitions/Watch" },
		watchOptions: { $ref: "#/definitions/WatchOptions" },
		builtins: {
			description: "Builtins features in rspack",
			type: "object",
			additionalProperties: !0
		}
	},
	t = Object.prototype.hasOwnProperty,
	a = new RegExp(
		"^(inline-|hidden-|eval-)?(nosources-)?(cheap-(module-)?)?source-map$",
		"u"
	);
function n(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: s,
		rootData: r = e
	} = {}
) {
	let i = null,
		o = 0;
	if (0 === o) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(n.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			let a;
			if (void 0 === e.import && (a = "import"))
				return (
					(n.errors = [
						{
							instancePath: t,
							schemaPath: "#/required",
							keyword: "required",
							params: { missingProperty: a },
							message: "must have required property '" + a + "'"
						}
					]),
					!1
				);
			{
				const a = o;
				for (const a in e)
					if ("import" !== a && "runtime" !== a)
						return (
							(n.errors = [
								{
									instancePath: t,
									schemaPath: "#/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: a },
									message: "must NOT have additional properties"
								}
							]),
							!1
						);
				if (a === o) {
					if (void 0 !== e.import) {
						let a = e.import;
						const s = o,
							r = o;
						let c = !1;
						const h = o;
						if (o === h)
							if (Array.isArray(a))
								if (a.length < 1) {
									const e = {
										instancePath: t + "/import",
										schemaPath: "#/definitions/EntryItem/anyOf/0/minItems",
										keyword: "minItems",
										params: { limit: 1 },
										message: "must NOT have fewer than 1 items"
									};
									null === i ? (i = [e]) : i.push(e), o++;
								} else {
									var l = !0;
									const e = a.length;
									for (let n = 0; n < e; n++) {
										let e = a[n];
										const s = o;
										if (o === s)
											if ("string" == typeof e) {
												if (e.length < 1) {
													const e = {
														instancePath: t + "/import/" + n,
														schemaPath:
															"#/definitions/EntryItem/anyOf/0/items/minLength",
														keyword: "minLength",
														params: {},
														message: 'must pass "minLength" keyword validation'
													};
													null === i ? (i = [e]) : i.push(e), o++;
												}
											} else {
												const e = {
													instancePath: t + "/import/" + n,
													schemaPath:
														"#/definitions/EntryItem/anyOf/0/items/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												null === i ? (i = [e]) : i.push(e), o++;
											}
										if (!(l = s === o)) break;
									}
									if (l) {
										let e,
											n = a.length;
										if (n > 1) {
											const s = {};
											for (; n--; ) {
												let r = a[n];
												if ("string" == typeof r) {
													if ("number" == typeof s[r]) {
														e = s[r];
														const a = {
															instancePath: t + "/import",
															schemaPath:
																"#/definitions/EntryItem/anyOf/0/uniqueItems",
															keyword: "uniqueItems",
															params: { i: n, j: e },
															message:
																"must NOT have duplicate items (items ## " +
																e +
																" and " +
																n +
																" are identical)"
														};
														null === i ? (i = [a]) : i.push(a), o++;
														break;
													}
													s[r] = n;
												}
											}
										}
									}
								}
							else {
								const e = {
									instancePath: t + "/import",
									schemaPath: "#/definitions/EntryItem/anyOf/0/type",
									keyword: "type",
									params: { type: "array" },
									message: "must be array"
								};
								null === i ? (i = [e]) : i.push(e), o++;
							}
						var p = h === o;
						if (((c = c || p), !c)) {
							const e = o;
							if (o === e)
								if ("string" == typeof a) {
									if (a.length < 1) {
										const e = {
											instancePath: t + "/import",
											schemaPath: "#/definitions/EntryItem/anyOf/1/minLength",
											keyword: "minLength",
											params: {},
											message: 'must pass "minLength" keyword validation'
										};
										null === i ? (i = [e]) : i.push(e), o++;
									}
								} else {
									const e = {
										instancePath: t + "/import",
										schemaPath: "#/definitions/EntryItem/anyOf/1/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									null === i ? (i = [e]) : i.push(e), o++;
								}
							(p = e === o), (c = c || p);
						}
						if (!c) {
							const e = {
								instancePath: t + "/import",
								schemaPath: "#/definitions/EntryItem/anyOf",
								keyword: "anyOf",
								params: {},
								message: "must match a schema in anyOf"
							};
							return (
								null === i ? (i = [e]) : i.push(e), o++, (n.errors = i), !1
							);
						}
						(o = r), null !== i && (r ? (i.length = r) : (i = null));
						var m = s === o;
					} else m = !0;
					if (m)
						if (void 0 !== e.runtime) {
							let a = e.runtime;
							const s = o,
								r = o;
							let l = !1;
							const p = o;
							if (!1 !== a) {
								const e = {
									instancePath: t + "/runtime",
									schemaPath: "#/definitions/EntryRuntime/anyOf/0/enum",
									keyword: "enum",
									params: {},
									message: 'must pass "enum" keyword validation'
								};
								null === i ? (i = [e]) : i.push(e), o++;
							}
							var c = p === o;
							if (((l = l || c), !l)) {
								const e = o;
								if (o === e)
									if ("string" == typeof a) {
										if (a.length < 1) {
											const e = {
												instancePath: t + "/runtime",
												schemaPath:
													"#/definitions/EntryRuntime/anyOf/1/minLength",
												keyword: "minLength",
												params: {},
												message: 'must pass "minLength" keyword validation'
											};
											null === i ? (i = [e]) : i.push(e), o++;
										}
									} else {
										const e = {
											instancePath: t + "/runtime",
											schemaPath: "#/definitions/EntryRuntime/anyOf/1/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										null === i ? (i = [e]) : i.push(e), o++;
									}
								(c = e === o), (l = l || c);
							}
							if (!l) {
								const e = {
									instancePath: t + "/runtime",
									schemaPath: "#/definitions/EntryRuntime/anyOf",
									keyword: "anyOf",
									params: {},
									message: "must match a schema in anyOf"
								};
								return (
									null === i ? (i = [e]) : i.push(e), o++, (n.errors = i), !1
								);
							}
							(o = r),
								null !== i && (r ? (i.length = r) : (i = null)),
								(m = s === o);
						} else m = !0;
				}
			}
		}
	}
	return (n.errors = i), 0 === o;
}
function s(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: r,
		rootData: i = e
	} = {}
) {
	let o = null,
		l = 0;
	if (0 === l) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(s.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		for (const a in e) {
			let r = e[a];
			const h = l,
				u = l;
			let f = !1;
			const y = l,
				d = l;
			let g = !1;
			const P = l;
			if (l === P)
				if (Array.isArray(r))
					if (r.length < 1) {
						const e = {
							instancePath:
								t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
							schemaPath: "#/definitions/EntryItem/anyOf/0/minItems",
							keyword: "minItems",
							params: { limit: 1 },
							message: "must NOT have fewer than 1 items"
						};
						null === o ? (o = [e]) : o.push(e), l++;
					} else {
						var p = !0;
						const e = r.length;
						for (let n = 0; n < e; n++) {
							let e = r[n];
							const s = l;
							if (l === s)
								if ("string" == typeof e) {
									if (e.length < 1) {
										const e = {
											instancePath:
												t +
												"/" +
												a.replace(/~/g, "~0").replace(/\//g, "~1") +
												"/" +
												n,
											schemaPath:
												"#/definitions/EntryItem/anyOf/0/items/minLength",
											keyword: "minLength",
											params: {},
											message: 'must pass "minLength" keyword validation'
										};
										null === o ? (o = [e]) : o.push(e), l++;
									}
								} else {
									const e = {
										instancePath:
											t +
											"/" +
											a.replace(/~/g, "~0").replace(/\//g, "~1") +
											"/" +
											n,
										schemaPath: "#/definitions/EntryItem/anyOf/0/items/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									null === o ? (o = [e]) : o.push(e), l++;
								}
							if (!(p = s === l)) break;
						}
						if (p) {
							let e,
								n = r.length;
							if (n > 1) {
								const s = {};
								for (; n--; ) {
									let i = r[n];
									if ("string" == typeof i) {
										if ("number" == typeof s[i]) {
											e = s[i];
											const r = {
												instancePath:
													t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
												schemaPath:
													"#/definitions/EntryItem/anyOf/0/uniqueItems",
												keyword: "uniqueItems",
												params: { i: n, j: e },
												message:
													"must NOT have duplicate items (items ## " +
													e +
													" and " +
													n +
													" are identical)"
											};
											null === o ? (o = [r]) : o.push(r), l++;
											break;
										}
										s[i] = n;
									}
								}
							}
						}
					}
				else {
					const e = {
						instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
						schemaPath: "#/definitions/EntryItem/anyOf/0/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					null === o ? (o = [e]) : o.push(e), l++;
				}
			var m = P === l;
			if (((g = g || m), !g)) {
				const e = l;
				if (l === e)
					if ("string" == typeof r) {
						if (r.length < 1) {
							const e = {
								instancePath:
									t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
								schemaPath: "#/definitions/EntryItem/anyOf/1/minLength",
								keyword: "minLength",
								params: {},
								message: 'must pass "minLength" keyword validation'
							};
							null === o ? (o = [e]) : o.push(e), l++;
						}
					} else {
						const e = {
							instancePath:
								t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
							schemaPath: "#/definitions/EntryItem/anyOf/1/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						};
						null === o ? (o = [e]) : o.push(e), l++;
					}
				(m = e === l), (g = g || m);
			}
			if (g) (l = d), null !== o && (d ? (o.length = d) : (o = null));
			else {
				const e = {
					instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
					schemaPath: "#/definitions/EntryItem/anyOf",
					keyword: "anyOf",
					params: {},
					message: "must match a schema in anyOf"
				};
				null === o ? (o = [e]) : o.push(e), l++;
			}
			var c = y === l;
			if (((f = f || c), !f)) {
				const s = l;
				n(r, {
					instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
					parentData: e,
					parentDataProperty: a,
					rootData: i
				}) ||
					((o = null === o ? n.errors : o.concat(n.errors)), (l = o.length)),
					(c = s === l),
					(f = f || c);
			}
			if (!f) {
				const e = {
					instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
					schemaPath: "#/additionalProperties/anyOf",
					keyword: "anyOf",
					params: {},
					message: "must match a schema in anyOf"
				};
				return null === o ? (o = [e]) : o.push(e), l++, (s.errors = o), !1;
			}
			if (((l = u), null !== o && (u ? (o.length = u) : (o = null)), h !== l))
				break;
		}
	}
	return (s.errors = o), 0 === l;
}
function r(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let i = null,
		o = 0;
	const l = o;
	let p = !1,
		m = null;
	const c = o,
		h = o;
	let u = !1;
	const f = o;
	if (o === f)
		if (Array.isArray(e))
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/EntryItem/anyOf/0/minItems",
					keyword: "minItems",
					params: { limit: 1 },
					message: "must NOT have fewer than 1 items"
				};
				null === i ? (i = [e]) : i.push(e), o++;
			} else {
				var y = !0;
				const a = e.length;
				for (let n = 0; n < a; n++) {
					let a = e[n];
					const s = o;
					if (o === s)
						if ("string" == typeof a) {
							if (a.length < 1) {
								const e = {
									instancePath: t + "/" + n,
									schemaPath: "#/definitions/EntryItem/anyOf/0/items/minLength",
									keyword: "minLength",
									params: {},
									message: 'must pass "minLength" keyword validation'
								};
								null === i ? (i = [e]) : i.push(e), o++;
							}
						} else {
							const e = {
								instancePath: t + "/" + n,
								schemaPath: "#/definitions/EntryItem/anyOf/0/items/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							};
							null === i ? (i = [e]) : i.push(e), o++;
						}
					if (!(y = s === o)) break;
				}
				if (y) {
					let a,
						n = e.length;
					if (n > 1) {
						const s = {};
						for (; n--; ) {
							let r = e[n];
							if ("string" == typeof r) {
								if ("number" == typeof s[r]) {
									a = s[r];
									const e = {
										instancePath: t,
										schemaPath: "#/definitions/EntryItem/anyOf/0/uniqueItems",
										keyword: "uniqueItems",
										params: { i: n, j: a },
										message:
											"must NOT have duplicate items (items ## " +
											a +
											" and " +
											n +
											" are identical)"
									};
									null === i ? (i = [e]) : i.push(e), o++;
									break;
								}
								s[r] = n;
							}
						}
					}
				}
			}
		else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/EntryItem/anyOf/0/type",
				keyword: "type",
				params: { type: "array" },
				message: "must be array"
			};
			null === i ? (i = [e]) : i.push(e), o++;
		}
	var d = f === o;
	if (((u = u || d), !u)) {
		const a = o;
		if (o === a)
			if ("string" == typeof e) {
				if (e.length < 1) {
					const e = {
						instancePath: t,
						schemaPath: "#/definitions/EntryItem/anyOf/1/minLength",
						keyword: "minLength",
						params: {},
						message: 'must pass "minLength" keyword validation'
					};
					null === i ? (i = [e]) : i.push(e), o++;
				}
			} else {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/EntryItem/anyOf/1/type",
					keyword: "type",
					params: { type: "string" },
					message: "must be string"
				};
				null === i ? (i = [e]) : i.push(e), o++;
			}
		(d = a === o), (u = u || d);
	}
	if (u) (o = h), null !== i && (h ? (i.length = h) : (i = null));
	else {
		const e = {
			instancePath: t,
			schemaPath: "#/definitions/EntryItem/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		null === i ? (i = [e]) : i.push(e), o++;
	}
	if ((c === o && ((p = !0), (m = 0)), !p)) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: m },
			message: "must match exactly one schema in oneOf"
		};
		return null === i ? (i = [e]) : i.push(e), o++, (r.errors = i), !1;
	}
	return (
		(o = l),
		null !== i && (l ? (i.length = l) : (i = null)),
		(r.errors = i),
		0 === o
	);
}
function i(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: o = e
	} = {}
) {
	let l = null,
		p = 0;
	const m = p;
	let c = !1;
	const h = p;
	s(e, {
		instancePath: t,
		parentData: a,
		parentDataProperty: n,
		rootData: o
	}) || ((l = null === l ? s.errors : l.concat(s.errors)), (p = l.length));
	var u = h === p;
	if (((c = c || u), !c)) {
		const s = p;
		r(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: o
		}) || ((l = null === l ? r.errors : l.concat(r.errors)), (p = l.length)),
			(u = s === p),
			(c = c || u);
	}
	if (!c) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === l ? (l = [e]) : l.push(e), p++, (i.errors = l), !1;
	}
	return (
		(p = m),
		null !== l && (m ? (l.length = m) : (l = null)),
		(i.errors = l),
		0 === p
	);
}
function o(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		l = 0;
	const p = l;
	let m = !1;
	const c = l;
	if (
		(i(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? i.errors : r.concat(i.errors)), (l = r.length)),
		(m = m || c === l),
		!m)
	) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), l++, (o.errors = r), !1;
	}
	return (
		(l = p),
		null !== r && (p ? (r.length = p) : (r = null)),
		(o.errors = r),
		0 === l
	);
}
function l(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let p = !1;
	const m = i;
	if (i === m)
		if (e && "object" == typeof e && !Array.isArray(e))
			for (const a in e) {
				const n = i,
					s = i;
				let o = !1;
				const l = i;
				if ("string" != typeof e[a]) {
					const e = {
						instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
						schemaPath: "#/definitions/ExternalItemValue/anyOf/0/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				if (((o = o || l === i), o))
					(i = s), null !== r && (s ? (r.length = s) : (r = null));
				else {
					const e = {
						instancePath: t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
						schemaPath: "#/definitions/ExternalItemValue/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				if (n !== i) break;
			}
		else {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/0/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	if (((p = p || m === i), !p)) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (l.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(l.errors = r),
		0 === i
	);
}
function p(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let m = !1;
	const c = i;
	if (
		(l(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? l.errors : r.concat(l.errors)), (i = r.length)),
		(m = m || c === i),
		!m)
	) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (p.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(p.errors = r),
		0 === i
	);
}
function m(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (i === p)
		if (Array.isArray(e)) {
			const a = e.length;
			for (let n = 0; n < a; n++) {
				let a = e[n];
				const s = i,
					o = i;
				let l = !1,
					p = null;
				const m = i,
					h = i;
				let u = !1;
				const f = i;
				if (!(a instanceof RegExp)) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/definitions/FilterItemTypes/anyOf/0/instanceof",
						keyword: "instanceof",
						params: {},
						message: 'must pass "instanceof" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				var c = f === i;
				if (((u = u || c), !u)) {
					const e = i;
					if ("string" != typeof a) {
						const e = {
							instancePath: t + "/" + n,
							schemaPath: "#/definitions/FilterItemTypes/anyOf/1/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
					if (((c = e === i), (u = u || c), !u)) {
						const e = i;
						if (!(a instanceof Function)) {
							const e = {
								instancePath: t + "/" + n,
								schemaPath: "#/definitions/FilterItemTypes/anyOf/2/instanceof",
								keyword: "instanceof",
								params: {},
								message: 'must pass "instanceof" keyword validation'
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						(c = e === i), (u = u || c);
					}
				}
				if (u) (i = h), null !== r && (h ? (r.length = h) : (r = null));
				else {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/definitions/FilterItemTypes/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				if ((m === i && ((l = !0), (p = 0)), l))
					(i = o), null !== r && (o ? (r.length = o) : (r = null));
				else {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/anyOf/0/items/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: p },
						message: "must match exactly one schema in oneOf"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				if (s !== i) break;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/0/type",
				keyword: "type",
				params: { type: "array" },
				message: "must be array"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var h = p === i;
	if (((l = l || h), !l)) {
		const a = i,
			n = i;
		let s = !1;
		const o = i;
		if (!(e instanceof RegExp)) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilterItemTypes/anyOf/0/instanceof",
				keyword: "instanceof",
				params: {},
				message: 'must pass "instanceof" keyword validation'
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		var u = o === i;
		if (((s = s || u), !s)) {
			const a = i;
			if ("string" != typeof e) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/FilterItemTypes/anyOf/1/type",
					keyword: "type",
					params: { type: "string" },
					message: "must be string"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
			if (((u = a === i), (s = s || u), !s)) {
				const a = i;
				if (!(e instanceof Function)) {
					const e = {
						instancePath: t,
						schemaPath: "#/definitions/FilterItemTypes/anyOf/2/instanceof",
						keyword: "instanceof",
						params: {},
						message: 'must pass "instanceof" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				(u = a === i), (s = s || u);
			}
		}
		if (s) (i = n), null !== r && (n ? (r.length = n) : (r = null));
		else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilterItemTypes/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(h = a === i), (l = l || h);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (m.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(m.errors = r),
		0 === i
	);
}
function c(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(c.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if (
					"appendOnly" !== a &&
					"colors" !== a &&
					"console" !== a &&
					"debug" !== a &&
					"level" !== a &&
					"stream" !== a
				)
					return (
						(c.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i) {
				if (void 0 !== e.appendOnly) {
					const a = i;
					if ("boolean" != typeof e.appendOnly)
						return (
							(c.errors = [
								{
									instancePath: t + "/appendOnly",
									schemaPath: "#/properties/appendOnly/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}
							]),
							!1
						);
					var o = a === i;
				} else o = !0;
				if (o) {
					if (void 0 !== e.colors) {
						const a = i;
						if ("boolean" != typeof e.colors)
							return (
								(c.errors = [
									{
										instancePath: t + "/colors",
										schemaPath: "#/properties/colors/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}
								]),
								!1
							);
						o = a === i;
					} else o = !0;
					if (o) {
						if (void 0 !== e.debug) {
							let a = e.debug;
							const n = i,
								p = i;
							let h = !1;
							const u = i;
							if ("boolean" != typeof a) {
								const e = {
									instancePath: t + "/debug",
									schemaPath: "#/properties/debug/anyOf/0/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
							var l = u === i;
							if (((h = h || l), !h)) {
								const n = i;
								m(a, {
									instancePath: t + "/debug",
									parentData: e,
									parentDataProperty: "debug",
									rootData: s
								}) ||
									((r = null === r ? m.errors : r.concat(m.errors)),
									(i = r.length)),
									(l = n === i),
									(h = h || l);
							}
							if (!h) {
								const e = {
									instancePath: t + "/debug",
									schemaPath: "#/properties/debug/anyOf",
									keyword: "anyOf",
									params: {},
									message: "must match a schema in anyOf"
								};
								return (
									null === r ? (r = [e]) : r.push(e), i++, (c.errors = r), !1
								);
							}
							(i = p),
								null !== r && (p ? (r.length = p) : (r = null)),
								(o = n === i);
						} else o = !0;
						if (o)
							if (void 0 !== e.level) {
								let a = e.level;
								const n = i;
								if (
									"none" !== a &&
									"error" !== a &&
									"warn" !== a &&
									"info" !== a &&
									"log" !== a &&
									"verbose" !== a
								)
									return (
										(c.errors = [
											{
												instancePath: t + "/level",
												schemaPath: "#/properties/level/enum",
												keyword: "enum",
												params: {},
												message: 'must pass "enum" keyword validation'
											}
										]),
										!1
									);
								o = n === i;
							} else o = !0;
					}
				}
			}
		}
	}
	return (c.errors = r), 0 === i;
}
const h = {
		exclude: {
			description: "Shortcut for resource.exclude.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		generator: {
			description: "The options for the module generator.",
			type: "object"
		},
		include: {
			description: "Shortcut for resource.include.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		issuer: {
			description:
				"Match the issuer of the module (The module pointing to this module).",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		oneOf: {
			description: "Only execute the first matching rule in this array.",
			type: "array",
			items: {
				description: "A rule.",
				oneOf: [{ $ref: "#/definitions/RuleSetRule" }]
			}
		},
		parser: {
			description: "Options for parsing.",
			type: "object",
			additionalProperties: !0
		},
		resolve: {
			description: "Options for the resolver.",
			type: "object",
			oneOf: [{ $ref: "#/definitions/ResolveOptions" }]
		},
		resource: {
			description: "Match the resource path of the module.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		resourceFragment: {
			description: "Match the resource fragment of the module.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		resourceQuery: {
			description: "Match the resource query of the module.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		rules: {
			description: "Match and execute these rules when this rule is matched.",
			type: "array",
			items: {
				description: "A rule.",
				oneOf: [{ $ref: "#/definitions/RuleSetRule" }]
			}
		},
		sideEffects: {
			description: "Flags a module as with or without side effects.",
			type: "boolean"
		},
		test: {
			description: "Shortcut for resource.test.",
			oneOf: [{ $ref: "#/definitions/RuleSetConditionOrConditions" }]
		},
		type: { description: "Module type to use for the module.", type: "string" },
		use: {
			description: "Modifiers applied to the module when rule is matched.",
			oneOf: [{ $ref: "#/definitions/RuleSetUse" }]
		}
	},
	u = { validate: d };
function f(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!Array.isArray(e))
			return (
				(f.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					}
				]),
				!1
			);
		{
			const a = e.length;
			for (let n = 0; n < a; n++) {
				const a = i,
					o = i;
				let l = !1,
					p = null;
				const m = i;
				if (
					(u.validate(e[n], {
						instancePath: t + "/" + n,
						parentData: e,
						parentDataProperty: n,
						rootData: s
					}) ||
						((r = null === r ? u.validate.errors : r.concat(u.validate.errors)),
						(i = r.length)),
					m === i && ((l = !0), (p = 0)),
					!l)
				) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/items/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: p },
						message: "must match exactly one schema in oneOf"
					};
					return null === r ? (r = [e]) : r.push(e), i++, (f.errors = r), !1;
				}
				if (((i = o), null !== r && (o ? (r.length = o) : (r = null)), a !== i))
					break;
			}
		}
	}
	return (f.errors = r), 0 === i;
}
function y(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(y.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if ("and" !== a && "not" !== a && "or" !== a)
					return (
						(y.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i) {
				if (void 0 !== e.and) {
					const a = i,
						n = i;
					let l = !1,
						p = null;
					const m = i;
					if (
						(f(e.and, {
							instancePath: t + "/and",
							parentData: e,
							parentDataProperty: "and",
							rootData: s
						}) ||
							((r = null === r ? f.errors : r.concat(f.errors)),
							(i = r.length)),
						m === i && ((l = !0), (p = 0)),
						!l)
					) {
						const e = {
							instancePath: t + "/and",
							schemaPath: "#/properties/and/oneOf",
							keyword: "oneOf",
							params: { passingSchemas: p },
							message: "must match exactly one schema in oneOf"
						};
						return null === r ? (r = [e]) : r.push(e), i++, (y.errors = r), !1;
					}
					(i = n), null !== r && (n ? (r.length = n) : (r = null));
					var o = a === i;
				} else o = !0;
				if (o) {
					if (void 0 !== e.not) {
						const a = i,
							n = i;
						let l = !1,
							p = null;
						const m = i;
						if (
							(u.validate(e.not, {
								instancePath: t + "/not",
								parentData: e,
								parentDataProperty: "not",
								rootData: s
							}) ||
								((r =
									null === r ? u.validate.errors : r.concat(u.validate.errors)),
								(i = r.length)),
							m === i && ((l = !0), (p = 0)),
							!l)
						) {
							const e = {
								instancePath: t + "/not",
								schemaPath: "#/properties/not/oneOf",
								keyword: "oneOf",
								params: { passingSchemas: p },
								message: "must match exactly one schema in oneOf"
							};
							return (
								null === r ? (r = [e]) : r.push(e), i++, (y.errors = r), !1
							);
						}
						(i = n),
							null !== r && (n ? (r.length = n) : (r = null)),
							(o = a === i);
					} else o = !0;
					if (o)
						if (void 0 !== e.or) {
							const a = i,
								n = i;
							let l = !1,
								p = null;
							const m = i;
							if (
								(f(e.or, {
									instancePath: t + "/or",
									parentData: e,
									parentDataProperty: "or",
									rootData: s
								}) ||
									((r = null === r ? f.errors : r.concat(f.errors)),
									(i = r.length)),
								m === i && ((l = !0), (p = 0)),
								!l)
							) {
								const e = {
									instancePath: t + "/or",
									schemaPath: "#/properties/or/oneOf",
									keyword: "oneOf",
									params: { passingSchemas: p },
									message: "must match exactly one schema in oneOf"
								};
								return (
									null === r ? (r = [e]) : r.push(e), i++, (y.errors = r), !1
								);
							}
							(i = n),
								null !== r && (n ? (r.length = n) : (r = null)),
								(o = a === i);
						} else o = !0;
				}
			}
		}
	}
	return (y.errors = r), 0 === i;
}
function d(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (!(e instanceof RegExp)) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf/0/instanceof",
			keyword: "instanceof",
			params: {},
			message: 'must pass "instanceof" keyword validation'
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	var m = p === i;
	if (((l = l || m), !l)) {
		const o = i;
		if ("string" != typeof e) {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/1/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		if (((m = o === i), (l = l || m), !l)) {
			const o = i;
			if (
				(y(e, {
					instancePath: t,
					parentData: a,
					parentDataProperty: n,
					rootData: s
				}) ||
					((r = null === r ? y.errors : r.concat(y.errors)), (i = r.length)),
				(m = o === i),
				(l = l || m),
				!l)
			) {
				const o = i;
				f(e, {
					instancePath: t,
					parentData: a,
					parentDataProperty: n,
					rootData: s
				}) ||
					((r = null === r ? f.errors : r.concat(f.errors)), (i = r.length)),
					(m = o === i),
					(l = l || m);
			}
		}
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (d.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(d.errors = r),
		0 === i
	);
}
function g(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	d(e, {
		instancePath: t,
		parentData: a,
		parentDataProperty: n,
		rootData: s
	}) || ((r = null === r ? d.errors : r.concat(d.errors)), (i = r.length));
	var m = p === i;
	if (((l = l || m), !l)) {
		const o = i;
		f(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? f.errors : r.concat(f.errors)), (i = r.length)),
			(m = o === i),
			(l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (g.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(g.errors = r),
		0 === i
	);
}
const P = {
	alias: { $ref: "#/definitions/ResolveAlias" },
	browserField: {
		description:
			"Fields in the description file (usually package.json) which are used to redirect requests inside the module.",
		type: "boolean"
	},
	conditionNames: {
		description: "Condition names for exports field entry point.",
		type: "array",
		items: {
			description: "Condition names for exports field entry point.",
			type: "string"
		}
	},
	extensions: {
		description:
			"Extensions added to the request when trying to find the file.",
		type: "array",
		items: {
			description:
				"Extension added to the request when trying to find the file.",
			type: "string"
		}
	},
	fallback: {
		description: "Redirect module requests when normal resolving fails.",
		oneOf: [{ $ref: "#/definitions/ResolveAlias" }]
	},
	mainFields: {
		description:
			"Field names from the description file (package.json) which are used to find the default entry point.",
		type: "array",
		items: {
			description:
				"Field name from the description file (package.json) which are used to find the default entry point.",
			anyOf: [
				{
					type: "array",
					items: {
						description:
							"Part of the field path from the description file (package.json) which are used to find the default entry point.",
						type: "string",
						minLength: 1
					}
				},
				{ type: "string", minLength: 1 }
			]
		}
	},
	mainFiles: {
		description:
			"Filenames used to find the default entry point if there is no description file or main field.",
		type: "array",
		items: {
			description:
				"Filename used to find the default entry point if there is no description file or main field.",
			type: "string",
			minLength: 1
		}
	},
	modules: {
		description: "Folder names or directory paths where to find modules.",
		type: "array",
		items: {
			description: "Folder name or directory path where to find modules.",
			type: "string",
			minLength: 1
		}
	},
	preferRelative: {
		description:
			"Prefer to resolve module requests as relative request and fallback to resolving as module.",
		type: "boolean"
	}
};
function b(
	e,
	{
		instancePath: a = "",
		parentData: n,
		parentDataProperty: s,
		rootData: r = e
	} = {}
) {
	let i = null,
		o = 0;
	if (0 === o) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(b.errors = [
					{
						instancePath: a,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const n = o;
			for (const n in e)
				if (!t.call(P, n))
					return (
						(b.errors = [
							{
								instancePath: a,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: n },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (n === o) {
				if (void 0 !== e.alias) {
					let t = e.alias;
					const n = o,
						s = o;
					let r = !1;
					const m = o;
					if (o === m)
						if (t && "object" == typeof t && !Array.isArray(t))
							for (const e in t) {
								let n = t[e];
								const s = o,
									r = o;
								let p = !1;
								const m = o;
								if (o === m)
									if (Array.isArray(n)) {
										const t = n.length;
										for (let s = 0; s < t; s++) {
											let t = n[s];
											const r = o;
											if (o === r)
												if ("string" == typeof t) {
													if (t.length < 1) {
														const t = {
															instancePath:
																a +
																"/alias/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1") +
																"/" +
																s,
															schemaPath:
																"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/items/minLength",
															keyword: "minLength",
															params: {},
															message:
																'must pass "minLength" keyword validation'
														};
														null === i ? (i = [t]) : i.push(t), o++;
													}
												} else {
													const t = {
														instancePath:
															a +
															"/alias/" +
															e.replace(/~/g, "~0").replace(/\//g, "~1") +
															"/" +
															s,
														schemaPath:
															"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													};
													null === i ? (i = [t]) : i.push(t), o++;
												}
											if (r !== o) break;
										}
									} else {
										const t = {
											instancePath:
												a +
												"/alias/" +
												e.replace(/~/g, "~0").replace(/\//g, "~1"),
											schemaPath:
												"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/type",
											keyword: "type",
											params: { type: "array" },
											message: "must be array"
										};
										null === i ? (i = [t]) : i.push(t), o++;
									}
								var l = m === o;
								if (((p = p || l), !p)) {
									const t = o;
									if (!1 !== n) {
										const t = {
											instancePath:
												a +
												"/alias/" +
												e.replace(/~/g, "~0").replace(/\//g, "~1"),
											schemaPath:
												"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/1/enum",
											keyword: "enum",
											params: {},
											message: 'must pass "enum" keyword validation'
										};
										null === i ? (i = [t]) : i.push(t), o++;
									}
									if (((l = t === o), (p = p || l), !p)) {
										const t = o;
										if (o === t)
											if ("string" == typeof n) {
												if (n.length < 1) {
													const t = {
														instancePath:
															a +
															"/alias/" +
															e.replace(/~/g, "~0").replace(/\//g, "~1"),
														schemaPath:
															"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/2/minLength",
														keyword: "minLength",
														params: {},
														message: 'must pass "minLength" keyword validation'
													};
													null === i ? (i = [t]) : i.push(t), o++;
												}
											} else {
												const t = {
													instancePath:
														a +
														"/alias/" +
														e.replace(/~/g, "~0").replace(/\//g, "~1"),
													schemaPath:
														"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/2/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												null === i ? (i = [t]) : i.push(t), o++;
											}
										(l = t === o), (p = p || l);
									}
								}
								if (p) (o = r), null !== i && (r ? (i.length = r) : (i = null));
								else {
									const t = {
										instancePath:
											a +
											"/alias/" +
											e.replace(/~/g, "~0").replace(/\//g, "~1"),
										schemaPath:
											"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf",
										keyword: "anyOf",
										params: {},
										message: "must match a schema in anyOf"
									};
									null === i ? (i = [t]) : i.push(t), o++;
								}
								if (s !== o) break;
							}
						else {
							const e = {
								instancePath: a + "/alias",
								schemaPath: "#/definitions/ResolveAlias/anyOf/0/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							};
							null === i ? (i = [e]) : i.push(e), o++;
						}
					if (((r = r || m === o), !r)) {
						const e = {
							instancePath: a + "/alias",
							schemaPath: "#/definitions/ResolveAlias/anyOf",
							keyword: "anyOf",
							params: {},
							message: "must match a schema in anyOf"
						};
						return null === i ? (i = [e]) : i.push(e), o++, (b.errors = i), !1;
					}
					(o = s), null !== i && (s ? (i.length = s) : (i = null));
					var p = n === o;
				} else p = !0;
				if (p) {
					if (void 0 !== e.browserField) {
						const t = o;
						if ("boolean" != typeof e.browserField)
							return (
								(b.errors = [
									{
										instancePath: a + "/browserField",
										schemaPath: "#/properties/browserField/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}
								]),
								!1
							);
						p = t === o;
					} else p = !0;
					if (p) {
						if (void 0 !== e.conditionNames) {
							let t = e.conditionNames;
							const n = o;
							if (o === n) {
								if (!Array.isArray(t))
									return (
										(b.errors = [
											{
												instancePath: a + "/conditionNames",
												schemaPath: "#/properties/conditionNames/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											}
										]),
										!1
									);
								{
									const e = t.length;
									for (let n = 0; n < e; n++) {
										const e = o;
										if ("string" != typeof t[n])
											return (
												(b.errors = [
													{
														instancePath: a + "/conditionNames/" + n,
														schemaPath:
															"#/properties/conditionNames/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}
												]),
												!1
											);
										if (e !== o) break;
									}
								}
							}
							p = n === o;
						} else p = !0;
						if (p) {
							if (void 0 !== e.extensions) {
								let t = e.extensions;
								const n = o;
								if (o === n) {
									if (!Array.isArray(t))
										return (
											(b.errors = [
												{
													instancePath: a + "/extensions",
													schemaPath: "#/properties/extensions/type",
													keyword: "type",
													params: { type: "array" },
													message: "must be array"
												}
											]),
											!1
										);
									{
										const e = t.length;
										for (let n = 0; n < e; n++) {
											const e = o;
											if ("string" != typeof t[n])
												return (
													(b.errors = [
														{
															instancePath: a + "/extensions/" + n,
															schemaPath: "#/properties/extensions/items/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														}
													]),
													!1
												);
											if (e !== o) break;
										}
									}
								}
								p = n === o;
							} else p = !0;
							if (p) {
								if (void 0 !== e.fallback) {
									let t = e.fallback;
									const n = o,
										s = o;
									let r = !1,
										l = null;
									const c = o,
										h = o;
									let u = !1;
									const f = o;
									if (o === f)
										if (t && "object" == typeof t && !Array.isArray(t))
											for (const e in t) {
												let n = t[e];
												const s = o,
													r = o;
												let l = !1;
												const p = o;
												if (o === p)
													if (Array.isArray(n)) {
														const t = n.length;
														for (let s = 0; s < t; s++) {
															let t = n[s];
															const r = o;
															if (o === r)
																if ("string" == typeof t) {
																	if (t.length < 1) {
																		const t = {
																			instancePath:
																				a +
																				"/fallback/" +
																				e
																					.replace(/~/g, "~0")
																					.replace(/\//g, "~1") +
																				"/" +
																				s,
																			schemaPath:
																				"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/items/minLength",
																			keyword: "minLength",
																			params: {},
																			message:
																				'must pass "minLength" keyword validation'
																		};
																		null === i ? (i = [t]) : i.push(t), o++;
																	}
																} else {
																	const t = {
																		instancePath:
																			a +
																			"/fallback/" +
																			e
																				.replace(/~/g, "~0")
																				.replace(/\//g, "~1") +
																			"/" +
																			s,
																		schemaPath:
																			"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/items/type",
																		keyword: "type",
																		params: { type: "string" },
																		message: "must be string"
																	};
																	null === i ? (i = [t]) : i.push(t), o++;
																}
															if (r !== o) break;
														}
													} else {
														const t = {
															instancePath:
																a +
																"/fallback/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1"),
															schemaPath:
																"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/0/type",
															keyword: "type",
															params: { type: "array" },
															message: "must be array"
														};
														null === i ? (i = [t]) : i.push(t), o++;
													}
												var m = p === o;
												if (((l = l || m), !l)) {
													const t = o;
													if (!1 !== n) {
														const t = {
															instancePath:
																a +
																"/fallback/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1"),
															schemaPath:
																"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/1/enum",
															keyword: "enum",
															params: {},
															message: 'must pass "enum" keyword validation'
														};
														null === i ? (i = [t]) : i.push(t), o++;
													}
													if (((m = t === o), (l = l || m), !l)) {
														const t = o;
														if (o === t)
															if ("string" == typeof n) {
																if (n.length < 1) {
																	const t = {
																		instancePath:
																			a +
																			"/fallback/" +
																			e
																				.replace(/~/g, "~0")
																				.replace(/\//g, "~1"),
																		schemaPath:
																			"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/2/minLength",
																		keyword: "minLength",
																		params: {},
																		message:
																			'must pass "minLength" keyword validation'
																	};
																	null === i ? (i = [t]) : i.push(t), o++;
																}
															} else {
																const t = {
																	instancePath:
																		a +
																		"/fallback/" +
																		e.replace(/~/g, "~0").replace(/\//g, "~1"),
																	schemaPath:
																		"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf/2/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																};
																null === i ? (i = [t]) : i.push(t), o++;
															}
														(m = t === o), (l = l || m);
													}
												}
												if (l)
													(o = r),
														null !== i && (r ? (i.length = r) : (i = null));
												else {
													const t = {
														instancePath:
															a +
															"/fallback/" +
															e.replace(/~/g, "~0").replace(/\//g, "~1"),
														schemaPath:
															"#/definitions/ResolveAlias/anyOf/0/additionalProperties/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													null === i ? (i = [t]) : i.push(t), o++;
												}
												if (s !== o) break;
											}
										else {
											const e = {
												instancePath: a + "/fallback",
												schemaPath: "#/definitions/ResolveAlias/anyOf/0/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											};
											null === i ? (i = [e]) : i.push(e), o++;
										}
									if (((u = u || f === o), u))
										(o = h), null !== i && (h ? (i.length = h) : (i = null));
									else {
										const e = {
											instancePath: a + "/fallback",
											schemaPath: "#/definitions/ResolveAlias/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										null === i ? (i = [e]) : i.push(e), o++;
									}
									if ((c === o && ((r = !0), (l = 0)), !r)) {
										const e = {
											instancePath: a + "/fallback",
											schemaPath: "#/properties/fallback/oneOf",
											keyword: "oneOf",
											params: { passingSchemas: l },
											message: "must match exactly one schema in oneOf"
										};
										return (
											null === i ? (i = [e]) : i.push(e),
											o++,
											(b.errors = i),
											!1
										);
									}
									(o = s),
										null !== i && (s ? (i.length = s) : (i = null)),
										(p = n === o);
								} else p = !0;
								if (p) {
									if (void 0 !== e.mainFields) {
										let t = e.mainFields;
										const n = o;
										if (o === n) {
											if (!Array.isArray(t))
												return (
													(b.errors = [
														{
															instancePath: a + "/mainFields",
															schemaPath: "#/properties/mainFields/type",
															keyword: "type",
															params: { type: "array" },
															message: "must be array"
														}
													]),
													!1
												);
											{
												const e = t.length;
												for (let n = 0; n < e; n++) {
													let e = t[n];
													const s = o,
														r = o;
													let l = !1;
													const p = o;
													if (o === p)
														if (Array.isArray(e)) {
															const t = e.length;
															for (let s = 0; s < t; s++) {
																let t = e[s];
																const r = o;
																if (o === r)
																	if ("string" == typeof t) {
																		if (t.length < 1) {
																			const e = {
																				instancePath:
																					a + "/mainFields/" + n + "/" + s,
																				schemaPath:
																					"#/properties/mainFields/items/anyOf/0/items/minLength",
																				keyword: "minLength",
																				params: {},
																				message:
																					'must pass "minLength" keyword validation'
																			};
																			null === i ? (i = [e]) : i.push(e), o++;
																		}
																	} else {
																		const e = {
																			instancePath:
																				a + "/mainFields/" + n + "/" + s,
																			schemaPath:
																				"#/properties/mainFields/items/anyOf/0/items/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		};
																		null === i ? (i = [e]) : i.push(e), o++;
																	}
																if (r !== o) break;
															}
														} else {
															const e = {
																instancePath: a + "/mainFields/" + n,
																schemaPath:
																	"#/properties/mainFields/items/anyOf/0/type",
																keyword: "type",
																params: { type: "array" },
																message: "must be array"
															};
															null === i ? (i = [e]) : i.push(e), o++;
														}
													var c = p === o;
													if (((l = l || c), !l)) {
														const t = o;
														if (o === t)
															if ("string" == typeof e) {
																if (e.length < 1) {
																	const e = {
																		instancePath: a + "/mainFields/" + n,
																		schemaPath:
																			"#/properties/mainFields/items/anyOf/1/minLength",
																		keyword: "minLength",
																		params: {},
																		message:
																			'must pass "minLength" keyword validation'
																	};
																	null === i ? (i = [e]) : i.push(e), o++;
																}
															} else {
																const e = {
																	instancePath: a + "/mainFields/" + n,
																	schemaPath:
																		"#/properties/mainFields/items/anyOf/1/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																};
																null === i ? (i = [e]) : i.push(e), o++;
															}
														(c = t === o), (l = l || c);
													}
													if (!l) {
														const e = {
															instancePath: a + "/mainFields/" + n,
															schemaPath: "#/properties/mainFields/items/anyOf",
															keyword: "anyOf",
															params: {},
															message: "must match a schema in anyOf"
														};
														return (
															null === i ? (i = [e]) : i.push(e),
															o++,
															(b.errors = i),
															!1
														);
													}
													if (
														((o = r),
														null !== i && (r ? (i.length = r) : (i = null)),
														s !== o)
													)
														break;
												}
											}
										}
										p = n === o;
									} else p = !0;
									if (p) {
										if (void 0 !== e.mainFiles) {
											let t = e.mainFiles;
											const n = o;
											if (o === n) {
												if (!Array.isArray(t))
													return (
														(b.errors = [
															{
																instancePath: a + "/mainFiles",
																schemaPath: "#/properties/mainFiles/type",
																keyword: "type",
																params: { type: "array" },
																message: "must be array"
															}
														]),
														!1
													);
												{
													const e = t.length;
													for (let n = 0; n < e; n++) {
														let e = t[n];
														const s = o;
														if (o === s) {
															if ("string" != typeof e)
																return (
																	(b.errors = [
																		{
																			instancePath: a + "/mainFiles/" + n,
																			schemaPath:
																				"#/properties/mainFiles/items/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		}
																	]),
																	!1
																);
															if (e.length < 1)
																return (
																	(b.errors = [
																		{
																			instancePath: a + "/mainFiles/" + n,
																			schemaPath:
																				"#/properties/mainFiles/items/minLength",
																			keyword: "minLength",
																			params: {},
																			message:
																				'must pass "minLength" keyword validation'
																		}
																	]),
																	!1
																);
														}
														if (s !== o) break;
													}
												}
											}
											p = n === o;
										} else p = !0;
										if (p) {
											if (void 0 !== e.modules) {
												let t = e.modules;
												const n = o;
												if (o === n) {
													if (!Array.isArray(t))
														return (
															(b.errors = [
																{
																	instancePath: a + "/modules",
																	schemaPath: "#/properties/modules/type",
																	keyword: "type",
																	params: { type: "array" },
																	message: "must be array"
																}
															]),
															!1
														);
													{
														const e = t.length;
														for (let n = 0; n < e; n++) {
															let e = t[n];
															const s = o;
															if (o === s) {
																if ("string" != typeof e)
																	return (
																		(b.errors = [
																			{
																				instancePath: a + "/modules/" + n,
																				schemaPath:
																					"#/properties/modules/items/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}
																		]),
																		!1
																	);
																if (e.length < 1)
																	return (
																		(b.errors = [
																			{
																				instancePath: a + "/modules/" + n,
																				schemaPath:
																					"#/properties/modules/items/minLength",
																				keyword: "minLength",
																				params: {},
																				message:
																					'must pass "minLength" keyword validation'
																			}
																		]),
																		!1
																	);
															}
															if (s !== o) break;
														}
													}
												}
												p = n === o;
											} else p = !0;
											if (p)
												if (void 0 !== e.preferRelative) {
													const t = o;
													if ("boolean" != typeof e.preferRelative)
														return (
															(b.errors = [
																{
																	instancePath: a + "/preferRelative",
																	schemaPath:
																		"#/properties/preferRelative/type",
																	keyword: "type",
																	params: { type: "boolean" },
																	message: "must be boolean"
																}
															]),
															!1
														);
													p = t === o;
												} else p = !0;
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (b.errors = i), 0 === o;
}
function k(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (i === p)
		if (e && "object" == typeof e && !Array.isArray(e)) {
			const a = i;
			for (const a in e)
				if ("loader" !== a && "options" !== a) {
					const e = {
						instancePath: t,
						schemaPath: "#/anyOf/0/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: a },
						message: "must NOT have additional properties"
					};
					null === r ? (r = [e]) : r.push(e), i++;
					break;
				}
			if (a === i) {
				if (void 0 !== e.loader) {
					let a = e.loader;
					const n = i,
						s = i;
					let o = !1,
						l = null;
					const p = i;
					if (i == i)
						if ("string" == typeof a) {
							if (a.length < 1) {
								const e = {
									instancePath: t + "/loader",
									schemaPath: "#/definitions/RuleSetLoader/minLength",
									keyword: "minLength",
									params: {},
									message: 'must pass "minLength" keyword validation'
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
						} else {
							const e = {
								instancePath: t + "/loader",
								schemaPath: "#/definitions/RuleSetLoader/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
					if ((p === i && ((o = !0), (l = 0)), o))
						(i = s), null !== r && (s ? (r.length = s) : (r = null));
					else {
						const e = {
							instancePath: t + "/loader",
							schemaPath: "#/anyOf/0/properties/loader/oneOf",
							keyword: "oneOf",
							params: { passingSchemas: l },
							message: "must match exactly one schema in oneOf"
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
					var m = n === i;
				} else m = !0;
				if (m)
					if (void 0 !== e.options) {
						let a = e.options;
						const n = i,
							s = i;
						let o = !1,
							l = null;
						const p = i,
							h = i;
						let u = !1;
						const f = i;
						if ("string" != typeof a) {
							const e = {
								instancePath: t + "/options",
								schemaPath: "#/definitions/RuleSetLoaderOptions/anyOf/0/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						var c = f === i;
						if (((u = u || c), !u)) {
							const e = i;
							if (!a || "object" != typeof a || Array.isArray(a)) {
								const e = {
									instancePath: t + "/options",
									schemaPath: "#/definitions/RuleSetLoaderOptions/anyOf/1/type",
									keyword: "type",
									params: { type: "object" },
									message: "must be object"
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
							(c = e === i), (u = u || c);
						}
						if (u) (i = h), null !== r && (h ? (r.length = h) : (r = null));
						else {
							const e = {
								instancePath: t + "/options",
								schemaPath: "#/definitions/RuleSetLoaderOptions/anyOf",
								keyword: "anyOf",
								params: {},
								message: "must match a schema in anyOf"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						if ((p === i && ((o = !0), (l = 0)), o))
							(i = s), null !== r && (s ? (r.length = s) : (r = null));
						else {
							const e = {
								instancePath: t + "/options",
								schemaPath: "#/anyOf/0/properties/options/oneOf",
								keyword: "oneOf",
								params: { passingSchemas: l },
								message: "must match exactly one schema in oneOf"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						m = n === i;
					} else m = !0;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/0/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var h = p === i;
	if (((l = l || h), !l)) {
		const a = i;
		if (i == i)
			if ("string" == typeof e) {
				if (e.length < 1) {
					const e = {
						instancePath: t,
						schemaPath: "#/definitions/RuleSetLoader/minLength",
						keyword: "minLength",
						params: {},
						message: 'must pass "minLength" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
			} else {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/RuleSetLoader/type",
					keyword: "type",
					params: { type: "string" },
					message: "must be string"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		(h = a === i), (l = l || h);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (k.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(k.errors = r),
		0 === i
	);
}
function O(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (i === p)
		if (Array.isArray(e)) {
			const a = e.length;
			for (let n = 0; n < a; n++) {
				const a = i,
					o = i;
				let l = !1,
					p = null;
				const m = i;
				if (
					(k(e[n], {
						instancePath: t + "/" + n,
						parentData: e,
						parentDataProperty: n,
						rootData: s
					}) ||
						((r = null === r ? k.errors : r.concat(k.errors)), (i = r.length)),
					m === i && ((l = !0), (p = 0)),
					l)
				)
					(i = o), null !== r && (o ? (r.length = o) : (r = null));
				else {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/anyOf/0/items/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: p },
						message: "must match exactly one schema in oneOf"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				if (a !== i) break;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/0/type",
				keyword: "type",
				params: { type: "array" },
				message: "must be array"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var m = p === i;
	if (((l = l || m), !l)) {
		const o = i;
		k(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? k.errors : r.concat(k.errors)), (i = r.length)),
			(m = o === i),
			(l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (O.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(O.errors = r),
		0 === i
	);
}
const w = { validate: v };
function v(
	e,
	{
		instancePath: a = "",
		parentData: n,
		parentDataProperty: s,
		rootData: r = e
	} = {}
) {
	let i = null,
		o = 0;
	if (0 === o) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(v.errors = [
					{
						instancePath: a,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const n = o;
			for (const n in e)
				if (!t.call(h, n))
					return (
						(v.errors = [
							{
								instancePath: a,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: n },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (n === o) {
				if (void 0 !== e.exclude) {
					const t = o,
						n = o;
					let s = !1,
						p = null;
					const m = o;
					if (
						(g(e.exclude, {
							instancePath: a + "/exclude",
							parentData: e,
							parentDataProperty: "exclude",
							rootData: r
						}) ||
							((i = null === i ? g.errors : i.concat(g.errors)),
							(o = i.length)),
						m === o && ((s = !0), (p = 0)),
						!s)
					) {
						const e = {
							instancePath: a + "/exclude",
							schemaPath: "#/properties/exclude/oneOf",
							keyword: "oneOf",
							params: { passingSchemas: p },
							message: "must match exactly one schema in oneOf"
						};
						return null === i ? (i = [e]) : i.push(e), o++, (v.errors = i), !1;
					}
					(o = n), null !== i && (n ? (i.length = n) : (i = null));
					var l = t === o;
				} else l = !0;
				if (l) {
					if (void 0 !== e.generator) {
						let t = e.generator;
						const n = o;
						if (!t || "object" != typeof t || Array.isArray(t))
							return (
								(v.errors = [
									{
										instancePath: a + "/generator",
										schemaPath: "#/properties/generator/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}
								]),
								!1
							);
						l = n === o;
					} else l = !0;
					if (l) {
						if (void 0 !== e.include) {
							const t = o,
								n = o;
							let s = !1,
								p = null;
							const m = o;
							if (
								(g(e.include, {
									instancePath: a + "/include",
									parentData: e,
									parentDataProperty: "include",
									rootData: r
								}) ||
									((i = null === i ? g.errors : i.concat(g.errors)),
									(o = i.length)),
								m === o && ((s = !0), (p = 0)),
								!s)
							) {
								const e = {
									instancePath: a + "/include",
									schemaPath: "#/properties/include/oneOf",
									keyword: "oneOf",
									params: { passingSchemas: p },
									message: "must match exactly one schema in oneOf"
								};
								return (
									null === i ? (i = [e]) : i.push(e), o++, (v.errors = i), !1
								);
							}
							(o = n),
								null !== i && (n ? (i.length = n) : (i = null)),
								(l = t === o);
						} else l = !0;
						if (l) {
							if (void 0 !== e.issuer) {
								const t = o,
									n = o;
								let s = !1,
									p = null;
								const m = o;
								if (
									(g(e.issuer, {
										instancePath: a + "/issuer",
										parentData: e,
										parentDataProperty: "issuer",
										rootData: r
									}) ||
										((i = null === i ? g.errors : i.concat(g.errors)),
										(o = i.length)),
									m === o && ((s = !0), (p = 0)),
									!s)
								) {
									const e = {
										instancePath: a + "/issuer",
										schemaPath: "#/properties/issuer/oneOf",
										keyword: "oneOf",
										params: { passingSchemas: p },
										message: "must match exactly one schema in oneOf"
									};
									return (
										null === i ? (i = [e]) : i.push(e), o++, (v.errors = i), !1
									);
								}
								(o = n),
									null !== i && (n ? (i.length = n) : (i = null)),
									(l = t === o);
							} else l = !0;
							if (l) {
								if (void 0 !== e.oneOf) {
									let t = e.oneOf;
									const n = o;
									if (o === n) {
										if (!Array.isArray(t))
											return (
												(v.errors = [
													{
														instancePath: a + "/oneOf",
														schemaPath: "#/properties/oneOf/type",
														keyword: "type",
														params: { type: "array" },
														message: "must be array"
													}
												]),
												!1
											);
										{
											const e = t.length;
											for (let n = 0; n < e; n++) {
												const e = o,
													s = o;
												let l = !1,
													p = null;
												const m = o;
												if (
													(w.validate(t[n], {
														instancePath: a + "/oneOf/" + n,
														parentData: t,
														parentDataProperty: n,
														rootData: r
													}) ||
														((i =
															null === i
																? w.validate.errors
																: i.concat(w.validate.errors)),
														(o = i.length)),
													m === o && ((l = !0), (p = 0)),
													!l)
												) {
													const e = {
														instancePath: a + "/oneOf/" + n,
														schemaPath: "#/properties/oneOf/items/oneOf",
														keyword: "oneOf",
														params: { passingSchemas: p },
														message: "must match exactly one schema in oneOf"
													};
													return (
														null === i ? (i = [e]) : i.push(e),
														o++,
														(v.errors = i),
														!1
													);
												}
												if (
													((o = s),
													null !== i && (s ? (i.length = s) : (i = null)),
													e !== o)
												)
													break;
											}
										}
									}
									l = n === o;
								} else l = !0;
								if (l) {
									if (void 0 !== e.parser) {
										let t = e.parser;
										const n = o;
										if (
											o === n &&
											(!t || "object" != typeof t || Array.isArray(t))
										)
											return (
												(v.errors = [
													{
														instancePath: a + "/parser",
														schemaPath: "#/properties/parser/type",
														keyword: "type",
														params: { type: "object" },
														message: "must be object"
													}
												]),
												!1
											);
										l = n === o;
									} else l = !0;
									if (l) {
										if (void 0 !== e.resolve) {
											let t = e.resolve;
											const n = o;
											if (!t || "object" != typeof t || Array.isArray(t))
												return (
													(v.errors = [
														{
															instancePath: a + "/resolve",
															schemaPath: "#/properties/resolve/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														}
													]),
													!1
												);
											const s = o;
											let p = !1,
												m = null;
											const c = o;
											if (
												(b(t, {
													instancePath: a + "/resolve",
													parentData: e,
													parentDataProperty: "resolve",
													rootData: r
												}) ||
													((i = null === i ? b.errors : i.concat(b.errors)),
													(o = i.length)),
												c === o && ((p = !0), (m = 0)),
												!p)
											) {
												const e = {
													instancePath: a + "/resolve",
													schemaPath: "#/properties/resolve/oneOf",
													keyword: "oneOf",
													params: { passingSchemas: m },
													message: "must match exactly one schema in oneOf"
												};
												return (
													null === i ? (i = [e]) : i.push(e),
													o++,
													(v.errors = i),
													!1
												);
											}
											(o = s),
												null !== i && (s ? (i.length = s) : (i = null)),
												(l = n === o);
										} else l = !0;
										if (l) {
											if (void 0 !== e.resource) {
												const t = o,
													n = o;
												let s = !1,
													p = null;
												const m = o;
												if (
													(g(e.resource, {
														instancePath: a + "/resource",
														parentData: e,
														parentDataProperty: "resource",
														rootData: r
													}) ||
														((i = null === i ? g.errors : i.concat(g.errors)),
														(o = i.length)),
													m === o && ((s = !0), (p = 0)),
													!s)
												) {
													const e = {
														instancePath: a + "/resource",
														schemaPath: "#/properties/resource/oneOf",
														keyword: "oneOf",
														params: { passingSchemas: p },
														message: "must match exactly one schema in oneOf"
													};
													return (
														null === i ? (i = [e]) : i.push(e),
														o++,
														(v.errors = i),
														!1
													);
												}
												(o = n),
													null !== i && (n ? (i.length = n) : (i = null)),
													(l = t === o);
											} else l = !0;
											if (l) {
												if (void 0 !== e.resourceFragment) {
													const t = o,
														n = o;
													let s = !1,
														p = null;
													const m = o;
													if (
														(g(e.resourceFragment, {
															instancePath: a + "/resourceFragment",
															parentData: e,
															parentDataProperty: "resourceFragment",
															rootData: r
														}) ||
															((i = null === i ? g.errors : i.concat(g.errors)),
															(o = i.length)),
														m === o && ((s = !0), (p = 0)),
														!s)
													) {
														const e = {
															instancePath: a + "/resourceFragment",
															schemaPath: "#/properties/resourceFragment/oneOf",
															keyword: "oneOf",
															params: { passingSchemas: p },
															message: "must match exactly one schema in oneOf"
														};
														return (
															null === i ? (i = [e]) : i.push(e),
															o++,
															(v.errors = i),
															!1
														);
													}
													(o = n),
														null !== i && (n ? (i.length = n) : (i = null)),
														(l = t === o);
												} else l = !0;
												if (l) {
													if (void 0 !== e.resourceQuery) {
														const t = o,
															n = o;
														let s = !1,
															p = null;
														const m = o;
														if (
															(g(e.resourceQuery, {
																instancePath: a + "/resourceQuery",
																parentData: e,
																parentDataProperty: "resourceQuery",
																rootData: r
															}) ||
																((i =
																	null === i ? g.errors : i.concat(g.errors)),
																(o = i.length)),
															m === o && ((s = !0), (p = 0)),
															!s)
														) {
															const e = {
																instancePath: a + "/resourceQuery",
																schemaPath: "#/properties/resourceQuery/oneOf",
																keyword: "oneOf",
																params: { passingSchemas: p },
																message:
																	"must match exactly one schema in oneOf"
															};
															return (
																null === i ? (i = [e]) : i.push(e),
																o++,
																(v.errors = i),
																!1
															);
														}
														(o = n),
															null !== i && (n ? (i.length = n) : (i = null)),
															(l = t === o);
													} else l = !0;
													if (l) {
														if (void 0 !== e.rules) {
															let t = e.rules;
															const n = o;
															if (o === n) {
																if (!Array.isArray(t))
																	return (
																		(v.errors = [
																			{
																				instancePath: a + "/rules",
																				schemaPath: "#/properties/rules/type",
																				keyword: "type",
																				params: { type: "array" },
																				message: "must be array"
																			}
																		]),
																		!1
																	);
																{
																	const e = t.length;
																	for (let n = 0; n < e; n++) {
																		const e = o,
																			s = o;
																		let l = !1,
																			p = null;
																		const m = o;
																		if (
																			(w.validate(t[n], {
																				instancePath: a + "/rules/" + n,
																				parentData: t,
																				parentDataProperty: n,
																				rootData: r
																			}) ||
																				((i =
																					null === i
																						? w.validate.errors
																						: i.concat(w.validate.errors)),
																				(o = i.length)),
																			m === o && ((l = !0), (p = 0)),
																			!l)
																		) {
																			const e = {
																				instancePath: a + "/rules/" + n,
																				schemaPath:
																					"#/properties/rules/items/oneOf",
																				keyword: "oneOf",
																				params: { passingSchemas: p },
																				message:
																					"must match exactly one schema in oneOf"
																			};
																			return (
																				null === i ? (i = [e]) : i.push(e),
																				o++,
																				(v.errors = i),
																				!1
																			);
																		}
																		if (
																			((o = s),
																			null !== i &&
																				(s ? (i.length = s) : (i = null)),
																			e !== o)
																		)
																			break;
																	}
																}
															}
															l = n === o;
														} else l = !0;
														if (l) {
															if (void 0 !== e.sideEffects) {
																const t = o;
																if ("boolean" != typeof e.sideEffects)
																	return (
																		(v.errors = [
																			{
																				instancePath: a + "/sideEffects",
																				schemaPath:
																					"#/properties/sideEffects/type",
																				keyword: "type",
																				params: { type: "boolean" },
																				message: "must be boolean"
																			}
																		]),
																		!1
																	);
																l = t === o;
															} else l = !0;
															if (l) {
																if (void 0 !== e.test) {
																	const t = o,
																		n = o;
																	let s = !1,
																		p = null;
																	const m = o;
																	if (
																		(g(e.test, {
																			instancePath: a + "/test",
																			parentData: e,
																			parentDataProperty: "test",
																			rootData: r
																		}) ||
																			((i =
																				null === i
																					? g.errors
																					: i.concat(g.errors)),
																			(o = i.length)),
																		m === o && ((s = !0), (p = 0)),
																		!s)
																	) {
																		const e = {
																			instancePath: a + "/test",
																			schemaPath: "#/properties/test/oneOf",
																			keyword: "oneOf",
																			params: { passingSchemas: p },
																			message:
																				"must match exactly one schema in oneOf"
																		};
																		return (
																			null === i ? (i = [e]) : i.push(e),
																			o++,
																			(v.errors = i),
																			!1
																		);
																	}
																	(o = n),
																		null !== i &&
																			(n ? (i.length = n) : (i = null)),
																		(l = t === o);
																} else l = !0;
																if (l) {
																	if (void 0 !== e.type) {
																		const t = o;
																		if ("string" != typeof e.type)
																			return (
																				(v.errors = [
																					{
																						instancePath: a + "/type",
																						schemaPath:
																							"#/properties/type/type",
																						keyword: "type",
																						params: { type: "string" },
																						message: "must be string"
																					}
																				]),
																				!1
																			);
																		l = t === o;
																	} else l = !0;
																	if (l)
																		if (void 0 !== e.use) {
																			const t = o,
																				n = o;
																			let s = !1,
																				p = null;
																			const m = o;
																			if (
																				(O(e.use, {
																					instancePath: a + "/use",
																					parentData: e,
																					parentDataProperty: "use",
																					rootData: r
																				}) ||
																					((i =
																						null === i
																							? O.errors
																							: i.concat(O.errors)),
																					(o = i.length)),
																				m === o && ((s = !0), (p = 0)),
																				!s)
																			) {
																				const e = {
																					instancePath: a + "/use",
																					schemaPath: "#/properties/use/oneOf",
																					keyword: "oneOf",
																					params: { passingSchemas: p },
																					message:
																						"must match exactly one schema in oneOf"
																				};
																				return (
																					null === i ? (i = [e]) : i.push(e),
																					o++,
																					(v.errors = i),
																					!1
																				);
																			}
																			(o = n),
																				null !== i &&
																					(n ? (i.length = n) : (i = null)),
																				(l = t === o);
																		} else l = !0;
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (v.errors = i), 0 === o;
}
function D(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!Array.isArray(e))
			return (
				(D.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					}
				]),
				!1
			);
		{
			const a = e.length;
			for (let n = 0; n < a; n++) {
				let a = e[n];
				const l = i,
					p = i;
				let m = !1;
				const c = i;
				if ("..." !== a) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/items/anyOf/0/enum",
						keyword: "enum",
						params: {},
						message: 'must pass "enum" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				var o = c === i;
				if (((m = m || o), !m)) {
					const l = i;
					v(a, {
						instancePath: t + "/" + n,
						parentData: e,
						parentDataProperty: n,
						rootData: s
					}) ||
						((r = null === r ? v.errors : r.concat(v.errors)), (i = r.length)),
						(o = l === i),
						(m = m || o);
				}
				if (!m) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/items/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					return null === r ? (r = [e]) : r.push(e), i++, (D.errors = r), !1;
				}
				if (((i = p), null !== r && (p ? (r.length = p) : (r = null)), l !== i))
					break;
			}
		}
	}
	return (D.errors = r), 0 === i;
}
function A(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(A.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if ("dataUrlCondition" !== a)
					return (
						(A.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i && void 0 !== e.dataUrlCondition) {
				let a = e.dataUrlCondition;
				const n = i;
				let s = !1;
				const o = i;
				if (i == i)
					if (a && "object" == typeof a && !Array.isArray(a)) {
						const e = i;
						for (const e in a)
							if ("maxSize" !== e) {
								const a = {
									instancePath: t + "/dataUrlCondition",
									schemaPath:
										"#/definitions/AssetParserDataUrlOptions/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: e },
									message: "must NOT have additional properties"
								};
								null === r ? (r = [a]) : r.push(a), i++;
								break;
							}
						if (e === i && void 0 !== a.maxSize) {
							let e = a.maxSize;
							if ("number" != typeof e || !isFinite(e)) {
								const e = {
									instancePath: t + "/dataUrlCondition/maxSize",
									schemaPath:
										"#/definitions/AssetParserDataUrlOptions/properties/maxSize/type",
									keyword: "type",
									params: { type: "number" },
									message: "must be number"
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
						}
					} else {
						const e = {
							instancePath: t + "/dataUrlCondition",
							schemaPath: "#/definitions/AssetParserDataUrlOptions/type",
							keyword: "type",
							params: { type: "object" },
							message: "must be object"
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
				if (((s = s || o === i), !s)) {
					const e = {
						instancePath: t + "/dataUrlCondition",
						schemaPath: "#/properties/dataUrlCondition/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					return null === r ? (r = [e]) : r.push(e), i++, (A.errors = r), !1;
				}
				(i = n), null !== r && (n ? (r.length = n) : (r = null));
			}
		}
	}
	return (A.errors = r), 0 === i;
}
function j(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(j.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if ("asset" !== a) {
					let n = e[a];
					const s = i;
					if (i === s && (!n || "object" != typeof n || Array.isArray(n)))
						return (
							(j.errors = [
								{
									instancePath:
										t + "/" + a.replace(/~/g, "~0").replace(/\//g, "~1"),
									schemaPath: "#/additionalProperties/type",
									keyword: "type",
									params: { type: "object" },
									message: "must be object"
								}
							]),
							!1
						);
					if (s !== i) break;
				}
			a === i &&
				void 0 !== e.asset &&
				(A(e.asset, {
					instancePath: t + "/asset",
					parentData: e,
					parentDataProperty: "asset",
					rootData: s
				}) ||
					((r = null === r ? A.errors : r.concat(A.errors)), (i = r.length)));
		}
	}
	return (j.errors = r), 0 === i;
}
function C(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(C.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if ("defaultRules" !== a && "parser" !== a && "rules" !== a)
					return (
						(C.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i) {
				if (void 0 !== e.defaultRules) {
					const a = i,
						n = i;
					let l = !1,
						p = null;
					const m = i;
					if (
						(D(e.defaultRules, {
							instancePath: t + "/defaultRules",
							parentData: e,
							parentDataProperty: "defaultRules",
							rootData: s
						}) ||
							((r = null === r ? D.errors : r.concat(D.errors)),
							(i = r.length)),
						m === i && ((l = !0), (p = 0)),
						!l)
					) {
						const e = {
							instancePath: t + "/defaultRules",
							schemaPath: "#/properties/defaultRules/oneOf",
							keyword: "oneOf",
							params: { passingSchemas: p },
							message: "must match exactly one schema in oneOf"
						};
						return null === r ? (r = [e]) : r.push(e), i++, (C.errors = r), !1;
					}
					(i = n), null !== r && (n ? (r.length = n) : (r = null));
					var o = a === i;
				} else o = !0;
				if (o) {
					if (void 0 !== e.parser) {
						const a = i;
						j(e.parser, {
							instancePath: t + "/parser",
							parentData: e,
							parentDataProperty: "parser",
							rootData: s
						}) ||
							((r = null === r ? j.errors : r.concat(j.errors)),
							(i = r.length)),
							(o = a === i);
					} else o = !0;
					if (o)
						if (void 0 !== e.rules) {
							const a = i,
								n = i;
							let l = !1,
								p = null;
							const m = i;
							if (
								(D(e.rules, {
									instancePath: t + "/rules",
									parentData: e,
									parentDataProperty: "rules",
									rootData: s
								}) ||
									((r = null === r ? D.errors : r.concat(D.errors)),
									(i = r.length)),
								m === i && ((l = !0), (p = 0)),
								!l)
							) {
								const e = {
									instancePath: t + "/rules",
									schemaPath: "#/properties/rules/oneOf",
									keyword: "oneOf",
									params: { passingSchemas: p },
									message: "must match exactly one schema in oneOf"
								};
								return (
									null === r ? (r = [e]) : r.push(e), i++, (C.errors = r), !1
								);
							}
							(i = n),
								null !== r && (n ? (r.length = n) : (r = null)),
								(o = a === i);
						} else o = !0;
				}
			}
		}
	}
	return (C.errors = r), 0 === i;
}
function L(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (!1 !== e) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf/0/enum",
			keyword: "enum",
			params: {},
			message: 'must pass "enum" keyword validation'
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	var m = p === i;
	if (((l = l || m), !l)) {
		const a = i;
		if (i == i)
			if (e && "object" == typeof e && !Array.isArray(e)) {
				const a = i;
				for (const a in e)
					if ("__dirname" !== a && "global" !== a) {
						const e = {
							instancePath: t,
							schemaPath: "#/definitions/NodeOptions/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: a },
							message: "must NOT have additional properties"
						};
						null === r ? (r = [e]) : r.push(e), i++;
						break;
					}
				if (a === i) {
					if (void 0 !== e.__dirname) {
						let a = e.__dirname;
						const n = i;
						if (
							!1 !== a &&
							!0 !== a &&
							"warn-mock" !== a &&
							"mock" !== a &&
							"eval-only" !== a
						) {
							const e = {
								instancePath: t + "/__dirname",
								schemaPath:
									"#/definitions/NodeOptions/properties/__dirname/enum",
								keyword: "enum",
								params: {},
								message: 'must pass "enum" keyword validation'
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						var c = n === i;
					} else c = !0;
					if (c)
						if (void 0 !== e.global) {
							let a = e.global;
							const n = i;
							if (!1 !== a && !0 !== a && "warn" !== a) {
								const e = {
									instancePath: t + "/global",
									schemaPath:
										"#/definitions/NodeOptions/properties/global/enum",
									keyword: "enum",
									params: {},
									message: 'must pass "enum" keyword validation'
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
							c = n === i;
						} else c = !0;
				}
			} else {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/NodeOptions/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		(m = a === i), (l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (L.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(L.errors = r),
		0 === i
	);
}
function x(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(x.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if (
					"cacheGroups" !== a &&
					"chunks" !== a &&
					"enforceSizeThreshold" !== a &&
					"maxAsyncRequests" !== a &&
					"maxInitialRequests" !== a &&
					"minChunks" !== a &&
					"minRemainingSize" !== a &&
					"minSize" !== a
				)
					return (
						(x.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i) {
				if (void 0 !== e.cacheGroups) {
					let a = e.cacheGroups;
					const n = i;
					if (i === n) {
						if (!a || "object" != typeof a || Array.isArray(a))
							return (
								(x.errors = [
									{
										instancePath: t + "/cacheGroups",
										schemaPath: "#/properties/cacheGroups/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									}
								]),
								!1
							);
						for (const e in a) {
							let n = a[e];
							const s = i,
								m = i;
							let c = !1;
							const h = i;
							if (i == i)
								if (n && "object" == typeof n && !Array.isArray(n)) {
									const a = i;
									for (const a in n)
										if (
											"chunks" !== a &&
											"minChunks" !== a &&
											"name" !== a &&
											"priority" !== a &&
											"reuseExistingChunk" !== a &&
											"test" !== a
										) {
											const n = {
												instancePath:
													t +
													"/cacheGroups/" +
													e.replace(/~/g, "~0").replace(/\//g, "~1"),
												schemaPath:
													"#/definitions/OptimizationSplitChunksCacheGroup/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: a },
												message: "must NOT have additional properties"
											};
											null === r ? (r = [n]) : r.push(n), i++;
											break;
										}
									if (a === i) {
										if (void 0 !== n.chunks) {
											let a = n.chunks;
											const s = i,
												p = i;
											let m = !1;
											const c = i;
											if ("initial" !== a && "async" !== a && "all" !== a) {
												const a = {
													instancePath:
														t +
														"/cacheGroups/" +
														e.replace(/~/g, "~0").replace(/\//g, "~1") +
														"/chunks",
													schemaPath:
														"#/definitions/OptimizationSplitChunksCacheGroup/properties/chunks/anyOf/0/enum",
													keyword: "enum",
													params: {},
													message: 'must pass "enum" keyword validation'
												};
												null === r ? (r = [a]) : r.push(a), i++;
											}
											var o = c === i;
											if (((m = m || o), !m)) {
												const n = i;
												if (!(a instanceof Function)) {
													const a = {
														instancePath:
															t +
															"/cacheGroups/" +
															e.replace(/~/g, "~0").replace(/\//g, "~1") +
															"/chunks",
														schemaPath:
															"#/definitions/OptimizationSplitChunksCacheGroup/properties/chunks/anyOf/1/instanceof",
														keyword: "instanceof",
														params: {},
														message: 'must pass "instanceof" keyword validation'
													};
													null === r ? (r = [a]) : r.push(a), i++;
												}
												(o = n === i), (m = m || o);
											}
											if (m)
												(i = p),
													null !== r && (p ? (r.length = p) : (r = null));
											else {
												const a = {
													instancePath:
														t +
														"/cacheGroups/" +
														e.replace(/~/g, "~0").replace(/\//g, "~1") +
														"/chunks",
													schemaPath:
														"#/definitions/OptimizationSplitChunksCacheGroup/properties/chunks/anyOf",
													keyword: "anyOf",
													params: {},
													message: "must match a schema in anyOf"
												};
												null === r ? (r = [a]) : r.push(a), i++;
											}
											var l = s === i;
										} else l = !0;
										if (l) {
											if (void 0 !== n.minChunks) {
												let a = n.minChunks;
												const s = i;
												if (i === s)
													if ("number" == typeof a && isFinite(a)) {
														if (a < 1 || isNaN(a)) {
															const a = {
																instancePath:
																	t +
																	"/cacheGroups/" +
																	e.replace(/~/g, "~0").replace(/\//g, "~1") +
																	"/minChunks",
																schemaPath:
																	"#/definitions/OptimizationSplitChunksCacheGroup/properties/minChunks/minimum",
																keyword: "minimum",
																params: { comparison: ">=", limit: 1 },
																message: "must be >= 1"
															};
															null === r ? (r = [a]) : r.push(a), i++;
														}
													} else {
														const a = {
															instancePath:
																t +
																"/cacheGroups/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1") +
																"/minChunks",
															schemaPath:
																"#/definitions/OptimizationSplitChunksCacheGroup/properties/minChunks/type",
															keyword: "type",
															params: { type: "number" },
															message: "must be number"
														};
														null === r ? (r = [a]) : r.push(a), i++;
													}
												l = s === i;
											} else l = !0;
											if (l) {
												if (void 0 !== n.name) {
													let a = n.name;
													const s = i,
														o = i;
													let m = !1;
													const c = i;
													if (!1 !== a) {
														const a = {
															instancePath:
																t +
																"/cacheGroups/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1") +
																"/name",
															schemaPath:
																"#/definitions/OptimizationSplitChunksCacheGroup/properties/name/anyOf/0/enum",
															keyword: "enum",
															params: {},
															message: 'must pass "enum" keyword validation'
														};
														null === r ? (r = [a]) : r.push(a), i++;
													}
													var p = c === i;
													if (((m = m || p), !m)) {
														const n = i;
														if ("string" != typeof a) {
															const a = {
																instancePath:
																	t +
																	"/cacheGroups/" +
																	e.replace(/~/g, "~0").replace(/\//g, "~1") +
																	"/name",
																schemaPath:
																	"#/definitions/OptimizationSplitChunksCacheGroup/properties/name/anyOf/1/type",
																keyword: "type",
																params: { type: "string" },
																message: "must be string"
															};
															null === r ? (r = [a]) : r.push(a), i++;
														}
														if (((p = n === i), (m = m || p), !m)) {
															const n = i;
															if (!(a instanceof Function)) {
																const a = {
																	instancePath:
																		t +
																		"/cacheGroups/" +
																		e.replace(/~/g, "~0").replace(/\//g, "~1") +
																		"/name",
																	schemaPath:
																		"#/definitions/OptimizationSplitChunksCacheGroup/properties/name/anyOf/2/instanceof",
																	keyword: "instanceof",
																	params: {},
																	message:
																		'must pass "instanceof" keyword validation'
																};
																null === r ? (r = [a]) : r.push(a), i++;
															}
															(p = n === i), (m = m || p);
														}
													}
													if (m)
														(i = o),
															null !== r && (o ? (r.length = o) : (r = null));
													else {
														const a = {
															instancePath:
																t +
																"/cacheGroups/" +
																e.replace(/~/g, "~0").replace(/\//g, "~1") +
																"/name",
															schemaPath:
																"#/definitions/OptimizationSplitChunksCacheGroup/properties/name/anyOf",
															keyword: "anyOf",
															params: {},
															message: "must match a schema in anyOf"
														};
														null === r ? (r = [a]) : r.push(a), i++;
													}
													l = s === i;
												} else l = !0;
												if (l) {
													if (void 0 !== n.priority) {
														let a = n.priority;
														const s = i;
														if ("number" != typeof a || !isFinite(a)) {
															const a = {
																instancePath:
																	t +
																	"/cacheGroups/" +
																	e.replace(/~/g, "~0").replace(/\//g, "~1") +
																	"/priority",
																schemaPath:
																	"#/definitions/OptimizationSplitChunksCacheGroup/properties/priority/type",
																keyword: "type",
																params: { type: "number" },
																message: "must be number"
															};
															null === r ? (r = [a]) : r.push(a), i++;
														}
														l = s === i;
													} else l = !0;
													if (l) {
														if (void 0 !== n.reuseExistingChunk) {
															const a = i;
															if ("boolean" != typeof n.reuseExistingChunk) {
																const a = {
																	instancePath:
																		t +
																		"/cacheGroups/" +
																		e.replace(/~/g, "~0").replace(/\//g, "~1") +
																		"/reuseExistingChunk",
																	schemaPath:
																		"#/definitions/OptimizationSplitChunksCacheGroup/properties/reuseExistingChunk/type",
																	keyword: "type",
																	params: { type: "boolean" },
																	message: "must be boolean"
																};
																null === r ? (r = [a]) : r.push(a), i++;
															}
															l = a === i;
														} else l = !0;
														if (l)
															if (void 0 !== n.test) {
																const a = i,
																	s = i;
																let o = !1;
																const p = i;
																if (!(n.test instanceof RegExp)) {
																	const a = {
																		instancePath:
																			t +
																			"/cacheGroups/" +
																			e
																				.replace(/~/g, "~0")
																				.replace(/\//g, "~1") +
																			"/test",
																		schemaPath:
																			"#/definitions/OptimizationSplitChunksCacheGroup/properties/test/anyOf/0/instanceof",
																		keyword: "instanceof",
																		params: {},
																		message:
																			'must pass "instanceof" keyword validation'
																	};
																	null === r ? (r = [a]) : r.push(a), i++;
																}
																if (((o = o || p === i), o))
																	(i = s),
																		null !== r &&
																			(s ? (r.length = s) : (r = null));
																else {
																	const a = {
																		instancePath:
																			t +
																			"/cacheGroups/" +
																			e
																				.replace(/~/g, "~0")
																				.replace(/\//g, "~1") +
																			"/test",
																		schemaPath:
																			"#/definitions/OptimizationSplitChunksCacheGroup/properties/test/anyOf",
																		keyword: "anyOf",
																		params: {},
																		message: "must match a schema in anyOf"
																	};
																	null === r ? (r = [a]) : r.push(a), i++;
																}
																l = a === i;
															} else l = !0;
													}
												}
											}
										}
									}
								} else {
									const a = {
										instancePath:
											t +
											"/cacheGroups/" +
											e.replace(/~/g, "~0").replace(/\//g, "~1"),
										schemaPath:
											"#/definitions/OptimizationSplitChunksCacheGroup/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									};
									null === r ? (r = [a]) : r.push(a), i++;
								}
							if (((c = c || h === i), !c)) {
								const a = {
									instancePath:
										t +
										"/cacheGroups/" +
										e.replace(/~/g, "~0").replace(/\//g, "~1"),
									schemaPath:
										"#/properties/cacheGroups/additionalProperties/anyOf",
									keyword: "anyOf",
									params: {},
									message: "must match a schema in anyOf"
								};
								return (
									null === r ? (r = [a]) : r.push(a), i++, (x.errors = r), !1
								);
							}
							if (
								((i = m),
								null !== r && (m ? (r.length = m) : (r = null)),
								s !== i)
							)
								break;
						}
					}
					var m = n === i;
				} else m = !0;
				if (m) {
					if (void 0 !== e.chunks) {
						let a = e.chunks;
						const n = i,
							s = i;
						let o = !1;
						const l = i;
						if ("initial" !== a && "async" !== a && "all" !== a) {
							const e = {
								instancePath: t + "/chunks",
								schemaPath: "#/properties/chunks/anyOf/0/enum",
								keyword: "enum",
								params: {},
								message: 'must pass "enum" keyword validation'
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						if (((o = o || l === i), !o)) {
							const e = {
								instancePath: t + "/chunks",
								schemaPath: "#/properties/chunks/anyOf",
								keyword: "anyOf",
								params: {},
								message: "must match a schema in anyOf"
							};
							return (
								null === r ? (r = [e]) : r.push(e), i++, (x.errors = r), !1
							);
						}
						(i = s),
							null !== r && (s ? (r.length = s) : (r = null)),
							(m = n === i);
					} else m = !0;
					if (m) {
						if (void 0 !== e.enforceSizeThreshold) {
							let a = e.enforceSizeThreshold;
							const n = i,
								s = i;
							let o = !1,
								l = null;
							const p = i,
								c = i;
							let h = !1;
							const u = i;
							if (i === u)
								if ("number" == typeof a && isFinite(a)) {
									if (a < 0 || isNaN(a)) {
										const e = {
											instancePath: t + "/enforceSizeThreshold",
											schemaPath:
												"#/definitions/OptimizationSplitChunksSizes/anyOf/0/minimum",
											keyword: "minimum",
											params: { comparison: ">=", limit: 0 },
											message: "must be >= 0"
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
								} else {
									const e = {
										instancePath: t + "/enforceSizeThreshold",
										schemaPath:
											"#/definitions/OptimizationSplitChunksSizes/anyOf/0/type",
										keyword: "type",
										params: { type: "number" },
										message: "must be number"
									};
									null === r ? (r = [e]) : r.push(e), i++;
								}
							if (((h = h || u === i), h))
								(i = c), null !== r && (c ? (r.length = c) : (r = null));
							else {
								const e = {
									instancePath: t + "/enforceSizeThreshold",
									schemaPath:
										"#/definitions/OptimizationSplitChunksSizes/anyOf",
									keyword: "anyOf",
									params: {},
									message: "must match a schema in anyOf"
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
							if ((p === i && ((o = !0), (l = 0)), !o)) {
								const e = {
									instancePath: t + "/enforceSizeThreshold",
									schemaPath: "#/properties/enforceSizeThreshold/oneOf",
									keyword: "oneOf",
									params: { passingSchemas: l },
									message: "must match exactly one schema in oneOf"
								};
								return (
									null === r ? (r = [e]) : r.push(e), i++, (x.errors = r), !1
								);
							}
							(i = s),
								null !== r && (s ? (r.length = s) : (r = null)),
								(m = n === i);
						} else m = !0;
						if (m) {
							if (void 0 !== e.maxAsyncRequests) {
								let a = e.maxAsyncRequests;
								const n = i;
								if (i === n) {
									if ("number" != typeof a || !isFinite(a))
										return (
											(x.errors = [
												{
													instancePath: t + "/maxAsyncRequests",
													schemaPath: "#/properties/maxAsyncRequests/type",
													keyword: "type",
													params: { type: "number" },
													message: "must be number"
												}
											]),
											!1
										);
									if (a < 1 || isNaN(a))
										return (
											(x.errors = [
												{
													instancePath: t + "/maxAsyncRequests",
													schemaPath: "#/properties/maxAsyncRequests/minimum",
													keyword: "minimum",
													params: { comparison: ">=", limit: 1 },
													message: "must be >= 1"
												}
											]),
											!1
										);
								}
								m = n === i;
							} else m = !0;
							if (m) {
								if (void 0 !== e.maxInitialRequests) {
									let a = e.maxInitialRequests;
									const n = i;
									if (i === n) {
										if ("number" != typeof a || !isFinite(a))
											return (
												(x.errors = [
													{
														instancePath: t + "/maxInitialRequests",
														schemaPath: "#/properties/maxInitialRequests/type",
														keyword: "type",
														params: { type: "number" },
														message: "must be number"
													}
												]),
												!1
											);
										if (a < 1 || isNaN(a))
											return (
												(x.errors = [
													{
														instancePath: t + "/maxInitialRequests",
														schemaPath:
															"#/properties/maxInitialRequests/minimum",
														keyword: "minimum",
														params: { comparison: ">=", limit: 1 },
														message: "must be >= 1"
													}
												]),
												!1
											);
									}
									m = n === i;
								} else m = !0;
								if (m) {
									if (void 0 !== e.minChunks) {
										let a = e.minChunks;
										const n = i;
										if (i === n) {
											if ("number" != typeof a || !isFinite(a))
												return (
													(x.errors = [
														{
															instancePath: t + "/minChunks",
															schemaPath: "#/properties/minChunks/type",
															keyword: "type",
															params: { type: "number" },
															message: "must be number"
														}
													]),
													!1
												);
											if (a < 1 || isNaN(a))
												return (
													(x.errors = [
														{
															instancePath: t + "/minChunks",
															schemaPath: "#/properties/minChunks/minimum",
															keyword: "minimum",
															params: { comparison: ">=", limit: 1 },
															message: "must be >= 1"
														}
													]),
													!1
												);
										}
										m = n === i;
									} else m = !0;
									if (m) {
										if (void 0 !== e.minRemainingSize) {
											let a = e.minRemainingSize;
											const n = i,
												s = i;
											let o = !1,
												l = null;
											const p = i,
												c = i;
											let h = !1;
											const u = i;
											if (i === u)
												if ("number" == typeof a && isFinite(a)) {
													if (a < 0 || isNaN(a)) {
														const e = {
															instancePath: t + "/minRemainingSize",
															schemaPath:
																"#/definitions/OptimizationSplitChunksSizes/anyOf/0/minimum",
															keyword: "minimum",
															params: { comparison: ">=", limit: 0 },
															message: "must be >= 0"
														};
														null === r ? (r = [e]) : r.push(e), i++;
													}
												} else {
													const e = {
														instancePath: t + "/minRemainingSize",
														schemaPath:
															"#/definitions/OptimizationSplitChunksSizes/anyOf/0/type",
														keyword: "type",
														params: { type: "number" },
														message: "must be number"
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
											if (((h = h || u === i), h))
												(i = c),
													null !== r && (c ? (r.length = c) : (r = null));
											else {
												const e = {
													instancePath: t + "/minRemainingSize",
													schemaPath:
														"#/definitions/OptimizationSplitChunksSizes/anyOf",
													keyword: "anyOf",
													params: {},
													message: "must match a schema in anyOf"
												};
												null === r ? (r = [e]) : r.push(e), i++;
											}
											if ((p === i && ((o = !0), (l = 0)), !o)) {
												const e = {
													instancePath: t + "/minRemainingSize",
													schemaPath: "#/properties/minRemainingSize/oneOf",
													keyword: "oneOf",
													params: { passingSchemas: l },
													message: "must match exactly one schema in oneOf"
												};
												return (
													null === r ? (r = [e]) : r.push(e),
													i++,
													(x.errors = r),
													!1
												);
											}
											(i = s),
												null !== r && (s ? (r.length = s) : (r = null)),
												(m = n === i);
										} else m = !0;
										if (m)
											if (void 0 !== e.minSize) {
												let a = e.minSize;
												const n = i,
													s = i;
												let o = !1,
													l = null;
												const p = i,
													c = i;
												let h = !1;
												const u = i;
												if (i === u)
													if ("number" == typeof a && isFinite(a)) {
														if (a < 0 || isNaN(a)) {
															const e = {
																instancePath: t + "/minSize",
																schemaPath:
																	"#/definitions/OptimizationSplitChunksSizes/anyOf/0/minimum",
																keyword: "minimum",
																params: { comparison: ">=", limit: 0 },
																message: "must be >= 0"
															};
															null === r ? (r = [e]) : r.push(e), i++;
														}
													} else {
														const e = {
															instancePath: t + "/minSize",
															schemaPath:
																"#/definitions/OptimizationSplitChunksSizes/anyOf/0/type",
															keyword: "type",
															params: { type: "number" },
															message: "must be number"
														};
														null === r ? (r = [e]) : r.push(e), i++;
													}
												if (((h = h || u === i), h))
													(i = c),
														null !== r && (c ? (r.length = c) : (r = null));
												else {
													const e = {
														instancePath: t + "/minSize",
														schemaPath:
															"#/definitions/OptimizationSplitChunksSizes/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
												if ((p === i && ((o = !0), (l = 0)), !o)) {
													const e = {
														instancePath: t + "/minSize",
														schemaPath: "#/properties/minSize/oneOf",
														keyword: "oneOf",
														params: { passingSchemas: l },
														message: "must match exactly one schema in oneOf"
													};
													return (
														null === r ? (r = [e]) : r.push(e),
														i++,
														(x.errors = r),
														!1
													);
												}
												(i = s),
													null !== r && (s ? (r.length = s) : (r = null)),
													(m = n === i);
											} else m = !0;
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (x.errors = r), 0 === i;
}
function S(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(S.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const a = i;
			for (const a in e)
				if (
					"chunkIds" !== a &&
					"minimize" !== a &&
					"minimizer" !== a &&
					"moduleIds" !== a &&
					"removeAvailableModules" !== a &&
					"runtimeChunk" !== a &&
					"sideEffects" !== a &&
					"splitChunks" !== a
				)
					return (
						(S.errors = [
							{
								instancePath: t,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (a === i) {
				if (void 0 !== e.chunkIds) {
					let a = e.chunkIds;
					const n = i;
					if ("named" !== a && "deterministic" !== a)
						return (
							(S.errors = [
								{
									instancePath: t + "/chunkIds",
									schemaPath: "#/properties/chunkIds/enum",
									keyword: "enum",
									params: {},
									message: 'must pass "enum" keyword validation'
								}
							]),
							!1
						);
					var o = n === i;
				} else o = !0;
				if (o) {
					if (void 0 !== e.minimize) {
						const a = i;
						if ("boolean" != typeof e.minimize)
							return (
								(S.errors = [
									{
										instancePath: t + "/minimize",
										schemaPath: "#/properties/minimize/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									}
								]),
								!1
							);
						o = a === i;
					} else o = !0;
					if (o) {
						if (void 0 !== e.minimizer) {
							let a = e.minimizer;
							const n = i;
							if (i === n) {
								if (!Array.isArray(a))
									return (
										(S.errors = [
											{
												instancePath: t + "/minimizer",
												schemaPath: "#/properties/minimizer/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											}
										]),
										!1
									);
								{
									const e = a.length;
									for (let n = 0; n < e; n++) {
										let e = a[n];
										const s = i,
											o = i;
										let p = !1;
										const m = i;
										if ("..." !== e) {
											const e = {
												instancePath: t + "/minimizer/" + n,
												schemaPath: "#/properties/minimizer/items/anyOf/0/enum",
												keyword: "enum",
												params: {},
												message: 'must pass "enum" keyword validation'
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
										var l = m === i;
										if (((p = p || l), !p)) {
											const a = i;
											if (i == i)
												if (e && "object" == typeof e && !Array.isArray(e)) {
													let a;
													if (void 0 === e.apply && (a = "apply")) {
														const e = {
															instancePath: t + "/minimizer/" + n,
															schemaPath:
																"#/definitions/RspackPluginInstance/required",
															keyword: "required",
															params: { missingProperty: a },
															message: "must have required property '" + a + "'"
														};
														null === r ? (r = [e]) : r.push(e), i++;
													} else if (
														void 0 !== e.apply &&
														!(e.apply instanceof Function)
													) {
														const e = {
															instancePath: t + "/minimizer/" + n + "/apply",
															schemaPath:
																"#/definitions/RspackPluginInstance/properties/apply/instanceof",
															keyword: "instanceof",
															params: {},
															message:
																'must pass "instanceof" keyword validation'
														};
														null === r ? (r = [e]) : r.push(e), i++;
													}
												} else {
													const e = {
														instancePath: t + "/minimizer/" + n,
														schemaPath:
															"#/definitions/RspackPluginInstance/type",
														keyword: "type",
														params: { type: "object" },
														message: "must be object"
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
											if (((l = a === i), (p = p || l), !p)) {
												const a = i;
												if (!(e instanceof Function)) {
													const e = {
														instancePath: t + "/minimizer/" + n,
														schemaPath:
															"#/definitions/RspackPluginFunction/instanceof",
														keyword: "instanceof",
														params: {},
														message: 'must pass "instanceof" keyword validation'
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
												(l = a === i), (p = p || l);
											}
										}
										if (!p) {
											const e = {
												instancePath: t + "/minimizer/" + n,
												schemaPath: "#/properties/minimizer/items/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											return (
												null === r ? (r = [e]) : r.push(e),
												i++,
												(S.errors = r),
												!1
											);
										}
										if (
											((i = o),
											null !== r && (o ? (r.length = o) : (r = null)),
											s !== i)
										)
											break;
									}
								}
							}
							o = n === i;
						} else o = !0;
						if (o) {
							if (void 0 !== e.moduleIds) {
								let a = e.moduleIds;
								const n = i;
								if ("named" !== a && "deterministic" !== a)
									return (
										(S.errors = [
											{
												instancePath: t + "/moduleIds",
												schemaPath: "#/properties/moduleIds/enum",
												keyword: "enum",
												params: {},
												message: 'must pass "enum" keyword validation'
											}
										]),
										!1
									);
								o = n === i;
							} else o = !0;
							if (o) {
								if (void 0 !== e.removeAvailableModules) {
									const a = i;
									if ("boolean" != typeof e.removeAvailableModules)
										return (
											(S.errors = [
												{
													instancePath: t + "/removeAvailableModules",
													schemaPath:
														"#/properties/removeAvailableModules/type",
													keyword: "type",
													params: { type: "boolean" },
													message: "must be boolean"
												}
											]),
											!1
										);
									o = a === i;
								} else o = !0;
								if (o) {
									if (void 0 !== e.runtimeChunk) {
										let a = e.runtimeChunk;
										const n = i,
											s = i;
										let l = !1;
										const c = i;
										if ("single" !== a && "multiple" !== a) {
											const e = {
												instancePath: t + "/runtimeChunk",
												schemaPath:
													"#/definitions/OptimizationRuntimeChunk/anyOf/0/enum",
												keyword: "enum",
												params: {},
												message: 'must pass "enum" keyword validation'
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
										var p = c === i;
										if (((l = l || p), !l)) {
											const e = i;
											if ("boolean" != typeof a) {
												const e = {
													instancePath: t + "/runtimeChunk",
													schemaPath:
														"#/definitions/OptimizationRuntimeChunk/anyOf/1/type",
													keyword: "type",
													params: { type: "boolean" },
													message: "must be boolean"
												};
												null === r ? (r = [e]) : r.push(e), i++;
											}
											if (((p = e === i), (l = l || p), !l)) {
												const e = i;
												if (i === e)
													if (a && "object" == typeof a && !Array.isArray(a)) {
														const e = i;
														for (const e in a)
															if ("name" !== e) {
																const a = {
																	instancePath: t + "/runtimeChunk",
																	schemaPath:
																		"#/definitions/OptimizationRuntimeChunk/anyOf/2/additionalProperties",
																	keyword: "additionalProperties",
																	params: { additionalProperty: e },
																	message: "must NOT have additional properties"
																};
																null === r ? (r = [a]) : r.push(a), i++;
																break;
															}
														if (e === i && void 0 !== a.name) {
															let e = a.name;
															const n = i;
															let s = !1;
															const o = i;
															if ("string" != typeof e) {
																const e = {
																	instancePath: t + "/runtimeChunk/name",
																	schemaPath:
																		"#/definitions/OptimizationRuntimeChunk/anyOf/2/properties/name/anyOf/0/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																};
																null === r ? (r = [e]) : r.push(e), i++;
															}
															var m = o === i;
															if (((s = s || m), !s)) {
																const a = i;
																if (!(e instanceof Function)) {
																	const e = {
																		instancePath: t + "/runtimeChunk/name",
																		schemaPath:
																			"#/definitions/OptimizationRuntimeChunk/anyOf/2/properties/name/anyOf/1/instanceof",
																		keyword: "instanceof",
																		params: {},
																		message:
																			'must pass "instanceof" keyword validation'
																	};
																	null === r ? (r = [e]) : r.push(e), i++;
																}
																(m = a === i), (s = s || m);
															}
															if (s)
																(i = n),
																	null !== r &&
																		(n ? (r.length = n) : (r = null));
															else {
																const e = {
																	instancePath: t + "/runtimeChunk/name",
																	schemaPath:
																		"#/definitions/OptimizationRuntimeChunk/anyOf/2/properties/name/anyOf",
																	keyword: "anyOf",
																	params: {},
																	message: "must match a schema in anyOf"
																};
																null === r ? (r = [e]) : r.push(e), i++;
															}
														}
													} else {
														const e = {
															instancePath: t + "/runtimeChunk",
															schemaPath:
																"#/definitions/OptimizationRuntimeChunk/anyOf/2/type",
															keyword: "type",
															params: { type: "object" },
															message: "must be object"
														};
														null === r ? (r = [e]) : r.push(e), i++;
													}
												(p = e === i), (l = l || p);
											}
										}
										if (!l) {
											const e = {
												instancePath: t + "/runtimeChunk",
												schemaPath:
													"#/definitions/OptimizationRuntimeChunk/anyOf",
												keyword: "anyOf",
												params: {},
												message: "must match a schema in anyOf"
											};
											return (
												null === r ? (r = [e]) : r.push(e),
												i++,
												(S.errors = r),
												!1
											);
										}
										(i = s),
											null !== r && (s ? (r.length = s) : (r = null)),
											(o = n === i);
									} else o = !0;
									if (o) {
										if (void 0 !== e.sideEffects) {
											let a = e.sideEffects;
											const n = i,
												s = i;
											let l = !1;
											const p = i;
											if ("flag" !== a) {
												const e = {
													instancePath: t + "/sideEffects",
													schemaPath: "#/properties/sideEffects/anyOf/0/enum",
													keyword: "enum",
													params: {},
													message: 'must pass "enum" keyword validation'
												};
												null === r ? (r = [e]) : r.push(e), i++;
											}
											var c = p === i;
											if (((l = l || c), !l)) {
												const e = i;
												if ("boolean" != typeof a) {
													const e = {
														instancePath: t + "/sideEffects",
														schemaPath: "#/properties/sideEffects/anyOf/1/type",
														keyword: "type",
														params: { type: "boolean" },
														message: "must be boolean"
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
												(c = e === i), (l = l || c);
											}
											if (!l) {
												const e = {
													instancePath: t + "/sideEffects",
													schemaPath: "#/properties/sideEffects/anyOf",
													keyword: "anyOf",
													params: {},
													message: "must match a schema in anyOf"
												};
												return (
													null === r ? (r = [e]) : r.push(e),
													i++,
													(S.errors = r),
													!1
												);
											}
											(i = s),
												null !== r && (s ? (r.length = s) : (r = null)),
												(o = n === i);
										} else o = !0;
										if (o)
											if (void 0 !== e.splitChunks) {
												let a = e.splitChunks;
												const n = i,
													l = i;
												let p = !1;
												const m = i;
												if (!1 !== a) {
													const e = {
														instancePath: t + "/splitChunks",
														schemaPath: "#/properties/splitChunks/anyOf/0/enum",
														keyword: "enum",
														params: {},
														message: 'must pass "enum" keyword validation'
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
												var h = m === i;
												if (((p = p || h), !p)) {
													const n = i;
													x(a, {
														instancePath: t + "/splitChunks",
														parentData: e,
														parentDataProperty: "splitChunks",
														rootData: s
													}) ||
														((r = null === r ? x.errors : r.concat(x.errors)),
														(i = r.length)),
														(h = n === i),
														(p = p || h);
												}
												if (!p) {
													const e = {
														instancePath: t + "/splitChunks",
														schemaPath: "#/properties/splitChunks/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													return (
														null === r ? (r = [e]) : r.push(e),
														i++,
														(S.errors = r),
														!1
													);
												}
												(i = l),
													null !== r && (l ? (r.length = l) : (r = null)),
													(o = n === i);
											} else o = !0;
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (S.errors = r), 0 === i;
}
const F = {
	assetModuleFilename: { $ref: "#/definitions/AssetModuleFilename" },
	auxiliaryComment: { oneOf: [{ $ref: "#/definitions/AuxiliaryComment" }] },
	chunkFilename: { $ref: "#/definitions/ChunkFilename" },
	cssChunkFilename: { $ref: "#/definitions/CssChunkFilename" },
	cssFilename: { $ref: "#/definitions/CssFilename" },
	enabledLibraryTypes: { $ref: "#/definitions/EnabledLibraryTypes" },
	filename: { $ref: "#/definitions/Filename" },
	library: { $ref: "#/definitions/Library" },
	libraryExport: { oneOf: [{ $ref: "#/definitions/LibraryExport" }] },
	libraryTarget: { oneOf: [{ $ref: "#/definitions/LibraryType" }] },
	module: { $ref: "#/definitions/OutputModule" },
	path: { $ref: "#/definitions/Path" },
	publicPath: { $ref: "#/definitions/PublicPath" },
	strictModuleErrorHandling: {
		$ref: "#/definitions/StrictModuleErrorHandling"
	},
	umdNamedDefine: { oneOf: [{ $ref: "#/definitions/UmdNamedDefine" }] },
	uniqueName: { $ref: "#/definitions/UniqueName" }
};
function T(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if ("string" != typeof e) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf/0/type",
			keyword: "type",
			params: { type: "string" },
			message: "must be string"
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	var m = p === i;
	if (((l = l || m), !l)) {
		const a = i;
		if (i == i)
			if (e && "object" == typeof e && !Array.isArray(e)) {
				const a = i;
				for (const a in e)
					if (
						"amd" !== a &&
						"commonjs" !== a &&
						"commonjs2" !== a &&
						"root" !== a
					) {
						const e = {
							instancePath: t,
							schemaPath:
								"#/definitions/LibraryCustomUmdCommentObject/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: a },
							message: "must NOT have additional properties"
						};
						null === r ? (r = [e]) : r.push(e), i++;
						break;
					}
				if (a === i) {
					if (void 0 !== e.amd) {
						const a = i;
						if ("string" != typeof e.amd) {
							const e = {
								instancePath: t + "/amd",
								schemaPath:
									"#/definitions/LibraryCustomUmdCommentObject/properties/amd/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
						var c = a === i;
					} else c = !0;
					if (c) {
						if (void 0 !== e.commonjs) {
							const a = i;
							if ("string" != typeof e.commonjs) {
								const e = {
									instancePath: t + "/commonjs",
									schemaPath:
										"#/definitions/LibraryCustomUmdCommentObject/properties/commonjs/type",
									keyword: "type",
									params: { type: "string" },
									message: "must be string"
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
							c = a === i;
						} else c = !0;
						if (c) {
							if (void 0 !== e.commonjs2) {
								const a = i;
								if ("string" != typeof e.commonjs2) {
									const e = {
										instancePath: t + "/commonjs2",
										schemaPath:
											"#/definitions/LibraryCustomUmdCommentObject/properties/commonjs2/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									null === r ? (r = [e]) : r.push(e), i++;
								}
								c = a === i;
							} else c = !0;
							if (c)
								if (void 0 !== e.root) {
									const a = i;
									if ("string" != typeof e.root) {
										const e = {
											instancePath: t + "/root",
											schemaPath:
												"#/definitions/LibraryCustomUmdCommentObject/properties/root/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
									c = a === i;
								} else c = !0;
						}
					}
				}
			} else {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/LibraryCustomUmdCommentObject/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		(m = a === i), (l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (T.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(T.errors = r),
		0 === i
	);
}
function R(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1,
		p = null;
	const m = i,
		c = i;
	let h = !1;
	const u = i;
	if (i === u)
		if ("string" == typeof e) {
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/FilenameTemplate/anyOf/0/minLength",
					keyword: "minLength",
					params: {},
					message: 'must pass "minLength" keyword validation'
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/0/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var f = u === i;
	if (((h = h || f), !h)) {
		const a = i;
		if (!(e instanceof Function)) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/1/instanceof",
				keyword: "instanceof",
				params: {},
				message: 'must pass "instanceof" keyword validation'
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(f = a === i), (h = h || f);
	}
	if (h) (i = c), null !== r && (c ? (r.length = c) : (r = null));
	else {
		const e = {
			instancePath: t,
			schemaPath: "#/definitions/FilenameTemplate/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	if ((m === i && ((l = !0), (p = 0)), !l)) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: p },
			message: "must match exactly one schema in oneOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (R.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(R.errors = r),
		0 === i
	);
}
function z(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1,
		p = null;
	const m = i,
		c = i;
	let h = !1;
	const u = i;
	if (i === u)
		if ("string" == typeof e) {
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/FilenameTemplate/anyOf/0/minLength",
					keyword: "minLength",
					params: {},
					message: 'must pass "minLength" keyword validation'
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/0/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var f = u === i;
	if (((h = h || f), !h)) {
		const a = i;
		if (!(e instanceof Function)) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/1/instanceof",
				keyword: "instanceof",
				params: {},
				message: 'must pass "instanceof" keyword validation'
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(f = a === i), (h = h || f);
	}
	if (h) (i = c), null !== r && (c ? (r.length = c) : (r = null));
	else {
		const e = {
			instancePath: t,
			schemaPath: "#/definitions/FilenameTemplate/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	if ((m === i && ((l = !0), (p = 0)), !l)) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: p },
			message: "must match exactly one schema in oneOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (z.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(z.errors = r),
		0 === i
	);
}
function E(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1,
		p = null;
	const m = i,
		c = i;
	let h = !1;
	const u = i;
	if (i === u)
		if ("string" == typeof e) {
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/FilenameTemplate/anyOf/0/minLength",
					keyword: "minLength",
					params: {},
					message: 'must pass "minLength" keyword validation'
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/0/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var f = u === i;
	if (((h = h || f), !h)) {
		const a = i;
		if (!(e instanceof Function)) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/1/instanceof",
				keyword: "instanceof",
				params: {},
				message: 'must pass "instanceof" keyword validation'
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(f = a === i), (h = h || f);
	}
	if (h) (i = c), null !== r && (c ? (r.length = c) : (r = null));
	else {
		const e = {
			instancePath: t,
			schemaPath: "#/definitions/FilenameTemplate/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	if ((m === i && ((l = !0), (p = 0)), !l)) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: p },
			message: "must match exactly one schema in oneOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (E.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(E.errors = r),
		0 === i
	);
}
function N(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!Array.isArray(e))
			return (
				(N.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					}
				]),
				!1
			);
		{
			const a = e.length;
			for (let n = 0; n < a; n++) {
				let a = e[n];
				const s = i,
					l = i;
				let p = !1;
				const m = i;
				if (
					"var" !== a &&
					"module" !== a &&
					"assign" !== a &&
					"assign-properties" !== a &&
					"this" !== a &&
					"window" !== a &&
					"self" !== a &&
					"global" !== a &&
					"commonjs" !== a &&
					"commonjs2" !== a &&
					"commonjs-module" !== a &&
					"commonjs-static" !== a &&
					"amd" !== a &&
					"amd-require" !== a &&
					"umd" !== a &&
					"umd2" !== a &&
					"jsonp" !== a &&
					"system" !== a
				) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/definitions/LibraryType/anyOf/0/enum",
						keyword: "enum",
						params: {},
						message: 'must pass "enum" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
				var o = m === i;
				if (((p = p || o), !p)) {
					const e = i;
					if ("string" != typeof a) {
						const e = {
							instancePath: t + "/" + n,
							schemaPath: "#/definitions/LibraryType/anyOf/1/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
					(o = e === i), (p = p || o);
				}
				if (!p) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/definitions/LibraryType/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					return null === r ? (r = [e]) : r.push(e), i++, (N.errors = r), !1;
				}
				if (((i = l), null !== r && (l ? (r.length = l) : (r = null)), s !== i))
					break;
			}
		}
	}
	return (N.errors = r), 0 === i;
}
function I(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1,
		p = null;
	const m = i,
		c = i;
	let h = !1;
	const u = i;
	if (i === u)
		if ("string" == typeof e) {
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/definitions/FilenameTemplate/anyOf/0/minLength",
					keyword: "minLength",
					params: {},
					message: 'must pass "minLength" keyword validation'
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		} else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/0/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var f = u === i;
	if (((h = h || f), !h)) {
		const a = i;
		if (!(e instanceof Function)) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/FilenameTemplate/anyOf/1/instanceof",
				keyword: "instanceof",
				params: {},
				message: 'must pass "instanceof" keyword validation'
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(f = a === i), (h = h || f);
	}
	if (h) (i = c), null !== r && (c ? (r.length = c) : (r = null));
	else {
		const e = {
			instancePath: t,
			schemaPath: "#/definitions/FilenameTemplate/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	if ((m === i && ((l = !0), (p = 0)), !l)) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: p },
			message: "must match exactly one schema in oneOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (I.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(I.errors = r),
		0 === i
	);
}
function $(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if (i === p)
		if (Array.isArray(e))
			if (e.length < 1) {
				const e = {
					instancePath: t,
					schemaPath: "#/anyOf/0/minItems",
					keyword: "minItems",
					params: { limit: 1 },
					message: "must NOT have fewer than 1 items"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			} else {
				const a = e.length;
				for (let n = 0; n < a; n++) {
					let a = e[n];
					const s = i;
					if (i === s)
						if ("string" == typeof a) {
							if (a.length < 1) {
								const e = {
									instancePath: t + "/" + n,
									schemaPath: "#/anyOf/0/items/minLength",
									keyword: "minLength",
									params: {},
									message: 'must pass "minLength" keyword validation'
								};
								null === r ? (r = [e]) : r.push(e), i++;
							}
						} else {
							const e = {
								instancePath: t + "/" + n,
								schemaPath: "#/anyOf/0/items/type",
								keyword: "type",
								params: { type: "string" },
								message: "must be string"
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
					if (s !== i) break;
				}
			}
		else {
			const e = {
				instancePath: t,
				schemaPath: "#/anyOf/0/type",
				keyword: "type",
				params: { type: "array" },
				message: "must be array"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
	var m = p === i;
	if (((l = l || m), !l)) {
		const a = i;
		if (i === a)
			if ("string" == typeof e) {
				if (e.length < 1) {
					const e = {
						instancePath: t,
						schemaPath: "#/anyOf/1/minLength",
						keyword: "minLength",
						params: {},
						message: 'must pass "minLength" keyword validation'
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
			} else {
				const e = {
					instancePath: t,
					schemaPath: "#/anyOf/1/type",
					keyword: "type",
					params: { type: "string" },
					message: "must be string"
				};
				null === r ? (r = [e]) : r.push(e), i++;
			}
		if (((m = a === i), (l = l || m), !l)) {
			const a = i;
			if (i == i)
				if (e && "object" == typeof e && !Array.isArray(e)) {
					const a = i;
					for (const a in e)
						if ("amd" !== a && "commonjs" !== a && "root" !== a) {
							const e = {
								instancePath: t,
								schemaPath:
									"#/definitions/LibraryCustomUmdObject/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							};
							null === r ? (r = [e]) : r.push(e), i++;
							break;
						}
					if (a === i) {
						if (void 0 !== e.amd) {
							let a = e.amd;
							const n = i;
							if (i === n)
								if ("string" == typeof a) {
									if (a.length < 1) {
										const e = {
											instancePath: t + "/amd",
											schemaPath:
												"#/definitions/LibraryCustomUmdObject/properties/amd/minLength",
											keyword: "minLength",
											params: {},
											message: 'must pass "minLength" keyword validation'
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
								} else {
									const e = {
										instancePath: t + "/amd",
										schemaPath:
											"#/definitions/LibraryCustomUmdObject/properties/amd/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									};
									null === r ? (r = [e]) : r.push(e), i++;
								}
							var c = n === i;
						} else c = !0;
						if (c) {
							if (void 0 !== e.commonjs) {
								let a = e.commonjs;
								const n = i;
								if (i === n)
									if ("string" == typeof a) {
										if (a.length < 1) {
											const e = {
												instancePath: t + "/commonjs",
												schemaPath:
													"#/definitions/LibraryCustomUmdObject/properties/commonjs/minLength",
												keyword: "minLength",
												params: {},
												message: 'must pass "minLength" keyword validation'
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
									} else {
										const e = {
											instancePath: t + "/commonjs",
											schemaPath:
												"#/definitions/LibraryCustomUmdObject/properties/commonjs/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
								c = n === i;
							} else c = !0;
							if (c)
								if (void 0 !== e.root) {
									let a = e.root;
									const n = i,
										s = i;
									let o = !1;
									const l = i;
									if (i === l)
										if (Array.isArray(a)) {
											const e = a.length;
											for (let n = 0; n < e; n++) {
												let e = a[n];
												const s = i;
												if (i === s)
													if ("string" == typeof e) {
														if (e.length < 1) {
															const e = {
																instancePath: t + "/root/" + n,
																schemaPath:
																	"#/definitions/LibraryCustomUmdObject/properties/root/anyOf/0/items/minLength",
																keyword: "minLength",
																params: {},
																message:
																	'must pass "minLength" keyword validation'
															};
															null === r ? (r = [e]) : r.push(e), i++;
														}
													} else {
														const e = {
															instancePath: t + "/root/" + n,
															schemaPath:
																"#/definitions/LibraryCustomUmdObject/properties/root/anyOf/0/items/type",
															keyword: "type",
															params: { type: "string" },
															message: "must be string"
														};
														null === r ? (r = [e]) : r.push(e), i++;
													}
												if (s !== i) break;
											}
										} else {
											const e = {
												instancePath: t + "/root",
												schemaPath:
													"#/definitions/LibraryCustomUmdObject/properties/root/anyOf/0/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
									var h = l === i;
									if (((o = o || h), !o)) {
										const e = i;
										if (i === e)
											if ("string" == typeof a) {
												if (a.length < 1) {
													const e = {
														instancePath: t + "/root",
														schemaPath:
															"#/definitions/LibraryCustomUmdObject/properties/root/anyOf/1/minLength",
														keyword: "minLength",
														params: {},
														message: 'must pass "minLength" keyword validation'
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
											} else {
												const e = {
													instancePath: t + "/root",
													schemaPath:
														"#/definitions/LibraryCustomUmdObject/properties/root/anyOf/1/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												null === r ? (r = [e]) : r.push(e), i++;
											}
										(h = e === i), (o = o || h);
									}
									if (o)
										(i = s), null !== r && (s ? (r.length = s) : (r = null));
									else {
										const e = {
											instancePath: t + "/root",
											schemaPath:
												"#/definitions/LibraryCustomUmdObject/properties/root/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
									c = n === i;
								} else c = !0;
						}
					}
				} else {
					const e = {
						instancePath: t,
						schemaPath: "#/definitions/LibraryCustomUmdObject/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					null === r ? (r = [e]) : r.push(e), i++;
				}
			(m = a === i), (l = l || m);
		}
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, ($.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		($.errors = r),
		0 === i
	);
}
function q(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(q.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			let a;
			if (void 0 === e.type && (a = "type"))
				return (
					(q.errors = [
						{
							instancePath: t,
							schemaPath: "#/required",
							keyword: "required",
							params: { missingProperty: a },
							message: "must have required property '" + a + "'"
						}
					]),
					!1
				);
			{
				const a = i;
				for (const a in e)
					if (
						"auxiliaryComment" !== a &&
						"export" !== a &&
						"name" !== a &&
						"type" !== a &&
						"umdNamedDefine" !== a
					)
						return (
							(q.errors = [
								{
									instancePath: t,
									schemaPath: "#/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: a },
									message: "must NOT have additional properties"
								}
							]),
							!1
						);
				if (a === i) {
					if (void 0 !== e.auxiliaryComment) {
						const a = i;
						T(e.auxiliaryComment, {
							instancePath: t + "/auxiliaryComment",
							parentData: e,
							parentDataProperty: "auxiliaryComment",
							rootData: s
						}) ||
							((r = null === r ? T.errors : r.concat(T.errors)),
							(i = r.length));
						var o = a === i;
					} else o = !0;
					if (o) {
						if (void 0 !== e.export) {
							let a = e.export;
							const n = i,
								s = i;
							let p = !1;
							const m = i;
							if (i === m)
								if (Array.isArray(a)) {
									const e = a.length;
									for (let n = 0; n < e; n++) {
										let e = a[n];
										const s = i;
										if (i === s)
											if ("string" == typeof e) {
												if (e.length < 1) {
													const e = {
														instancePath: t + "/export/" + n,
														schemaPath:
															"#/definitions/LibraryExport/anyOf/0/items/minLength",
														keyword: "minLength",
														params: {},
														message: 'must pass "minLength" keyword validation'
													};
													null === r ? (r = [e]) : r.push(e), i++;
												}
											} else {
												const e = {
													instancePath: t + "/export/" + n,
													schemaPath:
														"#/definitions/LibraryExport/anyOf/0/items/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												null === r ? (r = [e]) : r.push(e), i++;
											}
										if (s !== i) break;
									}
								} else {
									const e = {
										instancePath: t + "/export",
										schemaPath: "#/definitions/LibraryExport/anyOf/0/type",
										keyword: "type",
										params: { type: "array" },
										message: "must be array"
									};
									null === r ? (r = [e]) : r.push(e), i++;
								}
							var l = m === i;
							if (((p = p || l), !p)) {
								const e = i;
								if (i === e)
									if ("string" == typeof a) {
										if (a.length < 1) {
											const e = {
												instancePath: t + "/export",
												schemaPath:
													"#/definitions/LibraryExport/anyOf/1/minLength",
												keyword: "minLength",
												params: {},
												message: 'must pass "minLength" keyword validation'
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
									} else {
										const e = {
											instancePath: t + "/export",
											schemaPath: "#/definitions/LibraryExport/anyOf/1/type",
											keyword: "type",
											params: { type: "string" },
											message: "must be string"
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
								(l = e === i), (p = p || l);
							}
							if (!p) {
								const e = {
									instancePath: t + "/export",
									schemaPath: "#/definitions/LibraryExport/anyOf",
									keyword: "anyOf",
									params: {},
									message: "must match a schema in anyOf"
								};
								return (
									null === r ? (r = [e]) : r.push(e), i++, (q.errors = r), !1
								);
							}
							(i = s),
								null !== r && (s ? (r.length = s) : (r = null)),
								(o = n === i);
						} else o = !0;
						if (o) {
							if (void 0 !== e.name) {
								const a = i;
								$(e.name, {
									instancePath: t + "/name",
									parentData: e,
									parentDataProperty: "name",
									rootData: s
								}) ||
									((r = null === r ? $.errors : r.concat($.errors)),
									(i = r.length)),
									(o = a === i);
							} else o = !0;
							if (o) {
								if (void 0 !== e.type) {
									let a = e.type;
									const n = i,
										s = i;
									let l = !1;
									const m = i;
									if (
										"var" !== a &&
										"module" !== a &&
										"assign" !== a &&
										"assign-properties" !== a &&
										"this" !== a &&
										"window" !== a &&
										"self" !== a &&
										"global" !== a &&
										"commonjs" !== a &&
										"commonjs2" !== a &&
										"commonjs-module" !== a &&
										"commonjs-static" !== a &&
										"amd" !== a &&
										"amd-require" !== a &&
										"umd" !== a &&
										"umd2" !== a &&
										"jsonp" !== a &&
										"system" !== a
									) {
										const e = {
											instancePath: t + "/type",
											schemaPath: "#/definitions/LibraryType/anyOf/0/enum",
											keyword: "enum",
											params: {},
											message: 'must pass "enum" keyword validation'
										};
										null === r ? (r = [e]) : r.push(e), i++;
									}
									var p = m === i;
									if (((l = l || p), !l)) {
										const e = i;
										if ("string" != typeof a) {
											const e = {
												instancePath: t + "/type",
												schemaPath: "#/definitions/LibraryType/anyOf/1/type",
												keyword: "type",
												params: { type: "string" },
												message: "must be string"
											};
											null === r ? (r = [e]) : r.push(e), i++;
										}
										(p = e === i), (l = l || p);
									}
									if (!l) {
										const e = {
											instancePath: t + "/type",
											schemaPath: "#/definitions/LibraryType/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										return (
											null === r ? (r = [e]) : r.push(e),
											i++,
											(q.errors = r),
											!1
										);
									}
									(i = s),
										null !== r && (s ? (r.length = s) : (r = null)),
										(o = n === i);
								} else o = !0;
								if (o)
									if (void 0 !== e.umdNamedDefine) {
										const a = i;
										if ("boolean" != typeof e.umdNamedDefine)
											return (
												(q.errors = [
													{
														instancePath: t + "/umdNamedDefine",
														schemaPath: "#/definitions/UmdNamedDefine/type",
														keyword: "type",
														params: { type: "boolean" },
														message: "must be boolean"
													}
												]),
												!1
											);
										o = a === i;
									} else o = !0;
							}
						}
					}
				}
			}
		}
	}
	return (q.errors = r), 0 === i;
}
function G(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	$(e, {
		instancePath: t,
		parentData: a,
		parentDataProperty: n,
		rootData: s
	}) || ((r = null === r ? $.errors : r.concat($.errors)), (i = r.length));
	var m = p === i;
	if (((l = l || m), !l)) {
		const o = i;
		q(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? q.errors : r.concat(q.errors)), (i = r.length)),
			(m = o === i),
			(l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (G.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(G.errors = r),
		0 === i
	);
}
function U(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1;
	const p = i;
	if ("auto" !== e) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf/0/enum",
			keyword: "enum",
			params: {},
			message: 'must pass "enum" keyword validation'
		};
		null === r ? (r = [e]) : r.push(e), i++;
	}
	var m = p === i;
	if (((l = l || m), !l)) {
		const a = i,
			n = i;
		let s = !1;
		const o = i;
		if ("string" != typeof e) {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/RawPublicPath/anyOf/0/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		if (((s = s || o === i), s))
			(i = n), null !== r && (n ? (r.length = n) : (r = null));
		else {
			const e = {
				instancePath: t,
				schemaPath: "#/definitions/RawPublicPath/anyOf",
				keyword: "anyOf",
				params: {},
				message: "must match a schema in anyOf"
			};
			null === r ? (r = [e]) : r.push(e), i++;
		}
		(m = a === i), (l = l || m);
	}
	if (!l) {
		const e = {
			instancePath: t,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (U.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(U.errors = r),
		0 === i
	);
}
function M(
	e,
	{
		instancePath: a = "",
		parentData: n,
		parentDataProperty: s,
		rootData: r = e
	} = {}
) {
	let i = null,
		o = 0;
	if (0 === o) {
		if (!e || "object" != typeof e || Array.isArray(e))
			return (
				(M.errors = [
					{
						instancePath: a,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const n = o;
			for (const n in e)
				if (!t.call(F, n))
					return (
						(M.errors = [
							{
								instancePath: a,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: n },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (n === o) {
				if (void 0 !== e.assetModuleFilename) {
					const t = o,
						n = o;
					let s = !1;
					const r = o;
					if ("string" != typeof e.assetModuleFilename) {
						const e = {
							instancePath: a + "/assetModuleFilename",
							schemaPath: "#/definitions/AssetModuleFilename/anyOf/0/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						};
						null === i ? (i = [e]) : i.push(e), o++;
					}
					if (((s = s || r === o), !s)) {
						const e = {
							instancePath: a + "/assetModuleFilename",
							schemaPath: "#/definitions/AssetModuleFilename/anyOf",
							keyword: "anyOf",
							params: {},
							message: "must match a schema in anyOf"
						};
						return null === i ? (i = [e]) : i.push(e), o++, (M.errors = i), !1;
					}
					(o = n), null !== i && (n ? (i.length = n) : (i = null));
					var l = t === o;
				} else l = !0;
				if (l) {
					if (void 0 !== e.auxiliaryComment) {
						const t = o,
							n = o;
						let s = !1,
							p = null;
						const m = o;
						if (
							(T(e.auxiliaryComment, {
								instancePath: a + "/auxiliaryComment",
								parentData: e,
								parentDataProperty: "auxiliaryComment",
								rootData: r
							}) ||
								((i = null === i ? T.errors : i.concat(T.errors)),
								(o = i.length)),
							m === o && ((s = !0), (p = 0)),
							!s)
						) {
							const e = {
								instancePath: a + "/auxiliaryComment",
								schemaPath: "#/properties/auxiliaryComment/oneOf",
								keyword: "oneOf",
								params: { passingSchemas: p },
								message: "must match exactly one schema in oneOf"
							};
							return (
								null === i ? (i = [e]) : i.push(e), o++, (M.errors = i), !1
							);
						}
						(o = n),
							null !== i && (n ? (i.length = n) : (i = null)),
							(l = t === o);
					} else l = !0;
					if (l) {
						if (void 0 !== e.chunkFilename) {
							const t = o;
							R(e.chunkFilename, {
								instancePath: a + "/chunkFilename",
								parentData: e,
								parentDataProperty: "chunkFilename",
								rootData: r
							}) ||
								((i = null === i ? R.errors : i.concat(R.errors)),
								(o = i.length)),
								(l = t === o);
						} else l = !0;
						if (l) {
							if (void 0 !== e.cssChunkFilename) {
								const t = o;
								z(e.cssChunkFilename, {
									instancePath: a + "/cssChunkFilename",
									parentData: e,
									parentDataProperty: "cssChunkFilename",
									rootData: r
								}) ||
									((i = null === i ? z.errors : i.concat(z.errors)),
									(o = i.length)),
									(l = t === o);
							} else l = !0;
							if (l) {
								if (void 0 !== e.cssFilename) {
									const t = o;
									E(e.cssFilename, {
										instancePath: a + "/cssFilename",
										parentData: e,
										parentDataProperty: "cssFilename",
										rootData: r
									}) ||
										((i = null === i ? E.errors : i.concat(E.errors)),
										(o = i.length)),
										(l = t === o);
								} else l = !0;
								if (l) {
									if (void 0 !== e.enabledLibraryTypes) {
										const t = o;
										N(e.enabledLibraryTypes, {
											instancePath: a + "/enabledLibraryTypes",
											parentData: e,
											parentDataProperty: "enabledLibraryTypes",
											rootData: r
										}) ||
											((i = null === i ? N.errors : i.concat(N.errors)),
											(o = i.length)),
											(l = t === o);
									} else l = !0;
									if (l) {
										if (void 0 !== e.filename) {
											const t = o;
											I(e.filename, {
												instancePath: a + "/filename",
												parentData: e,
												parentDataProperty: "filename",
												rootData: r
											}) ||
												((i = null === i ? I.errors : i.concat(I.errors)),
												(o = i.length)),
												(l = t === o);
										} else l = !0;
										if (l) {
											if (void 0 !== e.library) {
												const t = o;
												G(e.library, {
													instancePath: a + "/library",
													parentData: e,
													parentDataProperty: "library",
													rootData: r
												}) ||
													((i = null === i ? G.errors : i.concat(G.errors)),
													(o = i.length)),
													(l = t === o);
											} else l = !0;
											if (l) {
												if (void 0 !== e.libraryExport) {
													let t = e.libraryExport;
													const n = o,
														s = o;
													let r = !1,
														m = null;
													const c = o,
														h = o;
													let u = !1;
													const f = o;
													if (o === f)
														if (Array.isArray(t)) {
															const e = t.length;
															for (let n = 0; n < e; n++) {
																let e = t[n];
																const s = o;
																if (o === s)
																	if ("string" == typeof e) {
																		if (e.length < 1) {
																			const e = {
																				instancePath: a + "/libraryExport/" + n,
																				schemaPath:
																					"#/definitions/LibraryExport/anyOf/0/items/minLength",
																				keyword: "minLength",
																				params: {},
																				message:
																					'must pass "minLength" keyword validation'
																			};
																			null === i ? (i = [e]) : i.push(e), o++;
																		}
																	} else {
																		const e = {
																			instancePath: a + "/libraryExport/" + n,
																			schemaPath:
																				"#/definitions/LibraryExport/anyOf/0/items/type",
																			keyword: "type",
																			params: { type: "string" },
																			message: "must be string"
																		};
																		null === i ? (i = [e]) : i.push(e), o++;
																	}
																if (s !== o) break;
															}
														} else {
															const e = {
																instancePath: a + "/libraryExport",
																schemaPath:
																	"#/definitions/LibraryExport/anyOf/0/type",
																keyword: "type",
																params: { type: "array" },
																message: "must be array"
															};
															null === i ? (i = [e]) : i.push(e), o++;
														}
													var p = f === o;
													if (((u = u || p), !u)) {
														const e = o;
														if (o === e)
															if ("string" == typeof t) {
																if (t.length < 1) {
																	const e = {
																		instancePath: a + "/libraryExport",
																		schemaPath:
																			"#/definitions/LibraryExport/anyOf/1/minLength",
																		keyword: "minLength",
																		params: {},
																		message:
																			'must pass "minLength" keyword validation'
																	};
																	null === i ? (i = [e]) : i.push(e), o++;
																}
															} else {
																const e = {
																	instancePath: a + "/libraryExport",
																	schemaPath:
																		"#/definitions/LibraryExport/anyOf/1/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																};
																null === i ? (i = [e]) : i.push(e), o++;
															}
														(p = e === o), (u = u || p);
													}
													if (u)
														(o = h),
															null !== i && (h ? (i.length = h) : (i = null));
													else {
														const e = {
															instancePath: a + "/libraryExport",
															schemaPath: "#/definitions/LibraryExport/anyOf",
															keyword: "anyOf",
															params: {},
															message: "must match a schema in anyOf"
														};
														null === i ? (i = [e]) : i.push(e), o++;
													}
													if ((c === o && ((r = !0), (m = 0)), !r)) {
														const e = {
															instancePath: a + "/libraryExport",
															schemaPath: "#/properties/libraryExport/oneOf",
															keyword: "oneOf",
															params: { passingSchemas: m },
															message: "must match exactly one schema in oneOf"
														};
														return (
															null === i ? (i = [e]) : i.push(e),
															o++,
															(M.errors = i),
															!1
														);
													}
													(o = s),
														null !== i && (s ? (i.length = s) : (i = null)),
														(l = n === o);
												} else l = !0;
												if (l) {
													if (void 0 !== e.libraryTarget) {
														let t = e.libraryTarget;
														const n = o,
															s = o;
														let r = !1,
															p = null;
														const c = o,
															h = o;
														let u = !1;
														const f = o;
														if (
															"var" !== t &&
															"module" !== t &&
															"assign" !== t &&
															"assign-properties" !== t &&
															"this" !== t &&
															"window" !== t &&
															"self" !== t &&
															"global" !== t &&
															"commonjs" !== t &&
															"commonjs2" !== t &&
															"commonjs-module" !== t &&
															"commonjs-static" !== t &&
															"amd" !== t &&
															"amd-require" !== t &&
															"umd" !== t &&
															"umd2" !== t &&
															"jsonp" !== t &&
															"system" !== t
														) {
															const e = {
																instancePath: a + "/libraryTarget",
																schemaPath:
																	"#/definitions/LibraryType/anyOf/0/enum",
																keyword: "enum",
																params: {},
																message: 'must pass "enum" keyword validation'
															};
															null === i ? (i = [e]) : i.push(e), o++;
														}
														var m = f === o;
														if (((u = u || m), !u)) {
															const e = o;
															if ("string" != typeof t) {
																const e = {
																	instancePath: a + "/libraryTarget",
																	schemaPath:
																		"#/definitions/LibraryType/anyOf/1/type",
																	keyword: "type",
																	params: { type: "string" },
																	message: "must be string"
																};
																null === i ? (i = [e]) : i.push(e), o++;
															}
															(m = e === o), (u = u || m);
														}
														if (u)
															(o = h),
																null !== i && (h ? (i.length = h) : (i = null));
														else {
															const e = {
																instancePath: a + "/libraryTarget",
																schemaPath: "#/definitions/LibraryType/anyOf",
																keyword: "anyOf",
																params: {},
																message: "must match a schema in anyOf"
															};
															null === i ? (i = [e]) : i.push(e), o++;
														}
														if ((c === o && ((r = !0), (p = 0)), !r)) {
															const e = {
																instancePath: a + "/libraryTarget",
																schemaPath: "#/properties/libraryTarget/oneOf",
																keyword: "oneOf",
																params: { passingSchemas: p },
																message:
																	"must match exactly one schema in oneOf"
															};
															return (
																null === i ? (i = [e]) : i.push(e),
																o++,
																(M.errors = i),
																!1
															);
														}
														(o = s),
															null !== i && (s ? (i.length = s) : (i = null)),
															(l = n === o);
													} else l = !0;
													if (l) {
														if (void 0 !== e.module) {
															const t = o;
															if ("boolean" != typeof e.module)
																return (
																	(M.errors = [
																		{
																			instancePath: a + "/module",
																			schemaPath:
																				"#/definitions/OutputModule/type",
																			keyword: "type",
																			params: { type: "boolean" },
																			message: "must be boolean"
																		}
																	]),
																	!1
																);
															l = t === o;
														} else l = !0;
														if (l) {
															if (void 0 !== e.path) {
																const t = o;
																if ("string" != typeof e.path)
																	return (
																		(M.errors = [
																			{
																				instancePath: a + "/path",
																				schemaPath: "#/definitions/Path/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			}
																		]),
																		!1
																	);
																l = t === o;
															} else l = !0;
															if (l) {
																if (void 0 !== e.publicPath) {
																	const t = o;
																	U(e.publicPath, {
																		instancePath: a + "/publicPath",
																		parentData: e,
																		parentDataProperty: "publicPath",
																		rootData: r
																	}) ||
																		((i =
																			null === i
																				? U.errors
																				: i.concat(U.errors)),
																		(o = i.length)),
																		(l = t === o);
																} else l = !0;
																if (l) {
																	if (void 0 !== e.strictModuleErrorHandling) {
																		const t = o;
																		if (
																			"boolean" !=
																			typeof e.strictModuleErrorHandling
																		)
																			return (
																				(M.errors = [
																					{
																						instancePath:
																							a + "/strictModuleErrorHandling",
																						schemaPath:
																							"#/definitions/StrictModuleErrorHandling/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					}
																				]),
																				!1
																			);
																		l = t === o;
																	} else l = !0;
																	if (l) {
																		if (void 0 !== e.umdNamedDefine) {
																			const t = o,
																				n = o;
																			let s = !1,
																				r = null;
																			const p = o;
																			if (
																				"boolean" != typeof e.umdNamedDefine
																			) {
																				const e = {
																					instancePath: a + "/umdNamedDefine",
																					schemaPath:
																						"#/definitions/UmdNamedDefine/type",
																					keyword: "type",
																					params: { type: "boolean" },
																					message: "must be boolean"
																				};
																				null === i ? (i = [e]) : i.push(e), o++;
																			}
																			if (
																				(p === o && ((s = !0), (r = 0)), !s)
																			) {
																				const e = {
																					instancePath: a + "/umdNamedDefine",
																					schemaPath:
																						"#/properties/umdNamedDefine/oneOf",
																					keyword: "oneOf",
																					params: { passingSchemas: r },
																					message:
																						"must match exactly one schema in oneOf"
																				};
																				return (
																					null === i ? (i = [e]) : i.push(e),
																					o++,
																					(M.errors = i),
																					!1
																				);
																			}
																			(o = n),
																				null !== i &&
																					(n ? (i.length = n) : (i = null)),
																				(l = t === o);
																		} else l = !0;
																		if (l)
																			if (void 0 !== e.uniqueName) {
																				let t = e.uniqueName;
																				const n = o;
																				if (o == o) {
																					if ("string" != typeof t)
																						return (
																							(M.errors = [
																								{
																									instancePath:
																										a + "/uniqueName",
																									schemaPath:
																										"#/definitions/UniqueName/type",
																									keyword: "type",
																									params: { type: "string" },
																									message: "must be string"
																								}
																							]),
																							!1
																						);
																					if (t.length < 1)
																						return (
																							(M.errors = [
																								{
																									instancePath:
																										a + "/uniqueName",
																									schemaPath:
																										"#/definitions/UniqueName/minLength",
																									keyword: "minLength",
																									params: {},
																									message:
																										'must pass "minLength" keyword validation'
																								}
																							]),
																							!1
																						);
																				}
																				l = n === o;
																			} else l = !0;
																	}
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (M.errors = i), 0 === o;
}
function W(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	if (0 === i) {
		if (!Array.isArray(e))
			return (
				(W.errors = [
					{
						instancePath: t,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					}
				]),
				!1
			);
		{
			const a = e.length;
			for (let n = 0; n < a; n++) {
				let a = e[n];
				const s = i,
					l = i;
				let p = !1;
				const m = i;
				if (i == i)
					if (a && "object" == typeof a && !Array.isArray(a)) {
						let e;
						if (void 0 === a.apply && (e = "apply")) {
							const a = {
								instancePath: t + "/" + n,
								schemaPath: "#/definitions/RspackPluginInstance/required",
								keyword: "required",
								params: { missingProperty: e },
								message: "must have required property '" + e + "'"
							};
							null === r ? (r = [a]) : r.push(a), i++;
						} else if (void 0 !== a.apply && !(a.apply instanceof Function)) {
							const e = {
								instancePath: t + "/" + n + "/apply",
								schemaPath:
									"#/definitions/RspackPluginInstance/properties/apply/instanceof",
								keyword: "instanceof",
								params: {},
								message: 'must pass "instanceof" keyword validation'
							};
							null === r ? (r = [e]) : r.push(e), i++;
						}
					} else {
						const e = {
							instancePath: t + "/" + n,
							schemaPath: "#/definitions/RspackPluginInstance/type",
							keyword: "type",
							params: { type: "object" },
							message: "must be object"
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
				var o = m === i;
				if (((p = p || o), !p)) {
					const e = i;
					if (!(a instanceof Function)) {
						const e = {
							instancePath: t + "/" + n,
							schemaPath: "#/definitions/RspackPluginFunction/instanceof",
							keyword: "instanceof",
							params: {},
							message: 'must pass "instanceof" keyword validation'
						};
						null === r ? (r = [e]) : r.push(e), i++;
					}
					(o = e === i), (p = p || o);
				}
				if (!p) {
					const e = {
						instancePath: t + "/" + n,
						schemaPath: "#/items/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					return null === r ? (r = [e]) : r.push(e), i++, (W.errors = r), !1;
				}
				if (((i = l), null !== r && (l ? (r.length = l) : (r = null)), s !== i))
					break;
			}
		}
	}
	return (W.errors = r), 0 === i;
}
function _(
	e,
	{
		instancePath: t = "",
		parentData: a,
		parentDataProperty: n,
		rootData: s = e
	} = {}
) {
	let r = null,
		i = 0;
	const o = i;
	let l = !1,
		p = null;
	const m = i;
	if (
		(b(e, {
			instancePath: t,
			parentData: a,
			parentDataProperty: n,
			rootData: s
		}) || ((r = null === r ? b.errors : r.concat(b.errors)), (i = r.length)),
		m === i && ((l = !0), (p = 0)),
		!l)
	) {
		const e = {
			instancePath: t,
			schemaPath: "#/oneOf",
			keyword: "oneOf",
			params: { passingSchemas: p },
			message: "must match exactly one schema in oneOf"
		};
		return null === r ? (r = [e]) : r.push(e), i++, (_.errors = r), !1;
	}
	return (
		(i = o),
		null !== r && (o ? (r.length = o) : (r = null)),
		(_.errors = r),
		0 === i
	);
}
const Q = {
	all: {
		description:
			"Fallback value for stats options when an option is not defined (has precedence over local rspack defaults).",
		type: "boolean"
	},
	assets: { description: "Add assets information.", type: "boolean" },
	chunkGroups: {
		description: "Display all chunk groups with the corresponding bundles.",
		type: "boolean"
	},
	chunks: { description: "Add chunk information.", type: "boolean" },
	colors: { description: "Enables/Disables colorful output.", type: "boolean" },
	entrypoints: {
		description: "Display the entry points with the corresponding bundles.",
		anyOf: [{ enum: ["auto"] }, { type: "boolean" }]
	},
	errors: { description: "Add errors.", type: "boolean" },
	errorsCount: { description: "Add errors count.", type: "boolean" },
	hash: { description: "Add the hash of the compilation.", type: "boolean" },
	modules: { description: "Add built modules information.", type: "boolean" },
	preset: {
		description: "Preset for the default values.",
		anyOf: [{ type: "boolean" }, { type: "string" }]
	},
	publicPath: { description: "Add public path information.", type: "boolean" },
	reasons: {
		description: "Add information about the reasons why modules are included.",
		type: "boolean"
	},
	warnings: { description: "Add warnings.", type: "boolean" },
	warningsCount: { description: "Add warnings count.", type: "boolean" }
};
function H(
	e,
	{
		instancePath: a = "",
		parentData: n,
		parentDataProperty: s,
		rootData: r = e
	} = {}
) {
	let i = null,
		o = 0;
	const l = o;
	let p = !1;
	const m = o;
	if (
		"none" !== e &&
		"errors-only" !== e &&
		"errors-warnings" !== e &&
		"normal" !== e &&
		"verbose" !== e
	) {
		const e = {
			instancePath: a,
			schemaPath: "#/anyOf/0/enum",
			keyword: "enum",
			params: {},
			message: 'must pass "enum" keyword validation'
		};
		null === i ? (i = [e]) : i.push(e), o++;
	}
	var c = m === o;
	if (((p = p || c), !p)) {
		const n = o;
		if ("boolean" != typeof e) {
			const e = {
				instancePath: a,
				schemaPath: "#/anyOf/1/type",
				keyword: "type",
				params: { type: "boolean" },
				message: "must be boolean"
			};
			null === i ? (i = [e]) : i.push(e), o++;
		}
		if (((c = n === o), (p = p || c), !p)) {
			const n = o;
			if (o == o)
				if (e && "object" == typeof e && !Array.isArray(e)) {
					const n = o;
					for (const n in e)
						if (!t.call(Q, n)) {
							const e = {
								instancePath: a,
								schemaPath: "#/definitions/StatsOptions/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: n },
								message: "must NOT have additional properties"
							};
							null === i ? (i = [e]) : i.push(e), o++;
							break;
						}
					if (n === o) {
						if (void 0 !== e.all) {
							const t = o;
							if ("boolean" != typeof e.all) {
								const e = {
									instancePath: a + "/all",
									schemaPath: "#/definitions/StatsOptions/properties/all/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								};
								null === i ? (i = [e]) : i.push(e), o++;
							}
							var h = t === o;
						} else h = !0;
						if (h) {
							if (void 0 !== e.assets) {
								const t = o;
								if ("boolean" != typeof e.assets) {
									const e = {
										instancePath: a + "/assets",
										schemaPath:
											"#/definitions/StatsOptions/properties/assets/type",
										keyword: "type",
										params: { type: "boolean" },
										message: "must be boolean"
									};
									null === i ? (i = [e]) : i.push(e), o++;
								}
								h = t === o;
							} else h = !0;
							if (h) {
								if (void 0 !== e.chunkGroups) {
									const t = o;
									if ("boolean" != typeof e.chunkGroups) {
										const e = {
											instancePath: a + "/chunkGroups",
											schemaPath:
												"#/definitions/StatsOptions/properties/chunkGroups/type",
											keyword: "type",
											params: { type: "boolean" },
											message: "must be boolean"
										};
										null === i ? (i = [e]) : i.push(e), o++;
									}
									h = t === o;
								} else h = !0;
								if (h) {
									if (void 0 !== e.chunks) {
										const t = o;
										if ("boolean" != typeof e.chunks) {
											const e = {
												instancePath: a + "/chunks",
												schemaPath:
													"#/definitions/StatsOptions/properties/chunks/type",
												keyword: "type",
												params: { type: "boolean" },
												message: "must be boolean"
											};
											null === i ? (i = [e]) : i.push(e), o++;
										}
										h = t === o;
									} else h = !0;
									if (h) {
										if (void 0 !== e.colors) {
											const t = o;
											if ("boolean" != typeof e.colors) {
												const e = {
													instancePath: a + "/colors",
													schemaPath:
														"#/definitions/StatsOptions/properties/colors/type",
													keyword: "type",
													params: { type: "boolean" },
													message: "must be boolean"
												};
												null === i ? (i = [e]) : i.push(e), o++;
											}
											h = t === o;
										} else h = !0;
										if (h) {
											if (void 0 !== e.entrypoints) {
												let t = e.entrypoints;
												const n = o,
													s = o;
												let r = !1;
												const l = o;
												if ("auto" !== t) {
													const e = {
														instancePath: a + "/entrypoints",
														schemaPath:
															"#/definitions/StatsOptions/properties/entrypoints/anyOf/0/enum",
														keyword: "enum",
														params: {},
														message: 'must pass "enum" keyword validation'
													};
													null === i ? (i = [e]) : i.push(e), o++;
												}
												var u = l === o;
												if (((r = r || u), !r)) {
													const e = o;
													if ("boolean" != typeof t) {
														const e = {
															instancePath: a + "/entrypoints",
															schemaPath:
																"#/definitions/StatsOptions/properties/entrypoints/anyOf/1/type",
															keyword: "type",
															params: { type: "boolean" },
															message: "must be boolean"
														};
														null === i ? (i = [e]) : i.push(e), o++;
													}
													(u = e === o), (r = r || u);
												}
												if (r)
													(o = s),
														null !== i && (s ? (i.length = s) : (i = null));
												else {
													const e = {
														instancePath: a + "/entrypoints",
														schemaPath:
															"#/definitions/StatsOptions/properties/entrypoints/anyOf",
														keyword: "anyOf",
														params: {},
														message: "must match a schema in anyOf"
													};
													null === i ? (i = [e]) : i.push(e), o++;
												}
												h = n === o;
											} else h = !0;
											if (h) {
												if (void 0 !== e.errors) {
													const t = o;
													if ("boolean" != typeof e.errors) {
														const e = {
															instancePath: a + "/errors",
															schemaPath:
																"#/definitions/StatsOptions/properties/errors/type",
															keyword: "type",
															params: { type: "boolean" },
															message: "must be boolean"
														};
														null === i ? (i = [e]) : i.push(e), o++;
													}
													h = t === o;
												} else h = !0;
												if (h) {
													if (void 0 !== e.errorsCount) {
														const t = o;
														if ("boolean" != typeof e.errorsCount) {
															const e = {
																instancePath: a + "/errorsCount",
																schemaPath:
																	"#/definitions/StatsOptions/properties/errorsCount/type",
																keyword: "type",
																params: { type: "boolean" },
																message: "must be boolean"
															};
															null === i ? (i = [e]) : i.push(e), o++;
														}
														h = t === o;
													} else h = !0;
													if (h) {
														if (void 0 !== e.hash) {
															const t = o;
															if ("boolean" != typeof e.hash) {
																const e = {
																	instancePath: a + "/hash",
																	schemaPath:
																		"#/definitions/StatsOptions/properties/hash/type",
																	keyword: "type",
																	params: { type: "boolean" },
																	message: "must be boolean"
																};
																null === i ? (i = [e]) : i.push(e), o++;
															}
															h = t === o;
														} else h = !0;
														if (h) {
															if (void 0 !== e.modules) {
																const t = o;
																if ("boolean" != typeof e.modules) {
																	const e = {
																		instancePath: a + "/modules",
																		schemaPath:
																			"#/definitions/StatsOptions/properties/modules/type",
																		keyword: "type",
																		params: { type: "boolean" },
																		message: "must be boolean"
																	};
																	null === i ? (i = [e]) : i.push(e), o++;
																}
																h = t === o;
															} else h = !0;
															if (h) {
																if (void 0 !== e.preset) {
																	let t = e.preset;
																	const n = o,
																		s = o;
																	let r = !1;
																	const l = o;
																	if ("boolean" != typeof t) {
																		const e = {
																			instancePath: a + "/preset",
																			schemaPath:
																				"#/definitions/StatsOptions/properties/preset/anyOf/0/type",
																			keyword: "type",
																			params: { type: "boolean" },
																			message: "must be boolean"
																		};
																		null === i ? (i = [e]) : i.push(e), o++;
																	}
																	var f = l === o;
																	if (((r = r || f), !r)) {
																		const e = o;
																		if ("string" != typeof t) {
																			const e = {
																				instancePath: a + "/preset",
																				schemaPath:
																					"#/definitions/StatsOptions/properties/preset/anyOf/1/type",
																				keyword: "type",
																				params: { type: "string" },
																				message: "must be string"
																			};
																			null === i ? (i = [e]) : i.push(e), o++;
																		}
																		(f = e === o), (r = r || f);
																	}
																	if (r)
																		(o = s),
																			null !== i &&
																				(s ? (i.length = s) : (i = null));
																	else {
																		const e = {
																			instancePath: a + "/preset",
																			schemaPath:
																				"#/definitions/StatsOptions/properties/preset/anyOf",
																			keyword: "anyOf",
																			params: {},
																			message: "must match a schema in anyOf"
																		};
																		null === i ? (i = [e]) : i.push(e), o++;
																	}
																	h = n === o;
																} else h = !0;
																if (h) {
																	if (void 0 !== e.publicPath) {
																		const t = o;
																		if ("boolean" != typeof e.publicPath) {
																			const e = {
																				instancePath: a + "/publicPath",
																				schemaPath:
																					"#/definitions/StatsOptions/properties/publicPath/type",
																				keyword: "type",
																				params: { type: "boolean" },
																				message: "must be boolean"
																			};
																			null === i ? (i = [e]) : i.push(e), o++;
																		}
																		h = t === o;
																	} else h = !0;
																	if (h) {
																		if (void 0 !== e.reasons) {
																			const t = o;
																			if ("boolean" != typeof e.reasons) {
																				const e = {
																					instancePath: a + "/reasons",
																					schemaPath:
																						"#/definitions/StatsOptions/properties/reasons/type",
																					keyword: "type",
																					params: { type: "boolean" },
																					message: "must be boolean"
																				};
																				null === i ? (i = [e]) : i.push(e), o++;
																			}
																			h = t === o;
																		} else h = !0;
																		if (h) {
																			if (void 0 !== e.warnings) {
																				const t = o;
																				if ("boolean" != typeof e.warnings) {
																					const e = {
																						instancePath: a + "/warnings",
																						schemaPath:
																							"#/definitions/StatsOptions/properties/warnings/type",
																						keyword: "type",
																						params: { type: "boolean" },
																						message: "must be boolean"
																					};
																					null === i ? (i = [e]) : i.push(e),
																						o++;
																				}
																				h = t === o;
																			} else h = !0;
																			if (h)
																				if (void 0 !== e.warningsCount) {
																					const t = o;
																					if (
																						"boolean" != typeof e.warningsCount
																					) {
																						const e = {
																							instancePath:
																								a + "/warningsCount",
																							schemaPath:
																								"#/definitions/StatsOptions/properties/warningsCount/type",
																							keyword: "type",
																							params: { type: "boolean" },
																							message: "must be boolean"
																						};
																						null === i ? (i = [e]) : i.push(e),
																							o++;
																					}
																					h = t === o;
																				} else h = !0;
																		}
																	}
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				} else {
					const e = {
						instancePath: a,
						schemaPath: "#/definitions/StatsOptions/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					null === i ? (i = [e]) : i.push(e), o++;
				}
			(c = n === o), (p = p || c);
		}
	}
	if (!p) {
		const e = {
			instancePath: a,
			schemaPath: "#/anyOf",
			keyword: "anyOf",
			params: {},
			message: "must match a schema in anyOf"
		};
		return null === i ? (i = [e]) : i.push(e), o++, (H.errors = i), !1;
	}
	return (
		(o = l),
		null !== i && (l ? (i.length = l) : (i = null)),
		(H.errors = i),
		0 === o
	);
}
function V(
	n,
	{
		instancePath: s = "",
		parentData: r,
		parentDataProperty: i,
		rootData: l = n
	} = {}
) {
	let m = null,
		h = 0;
	if (0 === h) {
		if (!n || "object" != typeof n || Array.isArray(n))
			return (
				(V.errors = [
					{
						instancePath: s,
						schemaPath: "#/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					}
				]),
				!1
			);
		{
			const r = h;
			for (const a in n)
				if (!t.call(e, a))
					return (
						(V.errors = [
							{
								instancePath: s,
								schemaPath: "#/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: a },
								message: "must NOT have additional properties"
							}
						]),
						!1
					);
			if (r === h) {
				if (void 0 !== n.cache) {
					const e = h;
					if ("boolean" != typeof n.cache)
						return (
							(V.errors = [
								{
									instancePath: s + "/cache",
									schemaPath: "#/definitions/CacheOptions/type",
									keyword: "type",
									params: { type: "boolean" },
									message: "must be boolean"
								}
							]),
							!1
						);
					var u = e === h;
				} else u = !0;
				if (u) {
					if (void 0 !== n.context) {
						const e = h;
						if ("string" != typeof n.context)
							return (
								(V.errors = [
									{
										instancePath: s + "/context",
										schemaPath: "#/definitions/Context/type",
										keyword: "type",
										params: { type: "string" },
										message: "must be string"
									}
								]),
								!1
							);
						u = e === h;
					} else u = !0;
					if (u) {
						if (void 0 !== n.dependencies) {
							let e = n.dependencies;
							const t = h;
							if (h == h) {
								if (!Array.isArray(e))
									return (
										(V.errors = [
											{
												instancePath: s + "/dependencies",
												schemaPath: "#/definitions/Dependencies/type",
												keyword: "type",
												params: { type: "array" },
												message: "must be array"
											}
										]),
										!1
									);
								{
									const t = e.length;
									for (let a = 0; a < t; a++) {
										const t = h;
										if ("string" != typeof e[a])
											return (
												(V.errors = [
													{
														instancePath: s + "/dependencies/" + a,
														schemaPath: "#/definitions/Dependencies/items/type",
														keyword: "type",
														params: { type: "string" },
														message: "must be string"
													}
												]),
												!1
											);
										if (t !== h) break;
									}
								}
							}
							u = t === h;
						} else u = !0;
						if (u) {
							if (void 0 !== n.devServer) {
								let e = n.devServer;
								const t = h;
								if (!e || "object" != typeof e || Array.isArray(e))
									return (
										(V.errors = [
											{
												instancePath: s + "/devServer",
												schemaPath: "#/definitions/DevServer/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											}
										]),
										!1
									);
								u = t === h;
							} else u = !0;
							if (u) {
								if (void 0 !== n.devtool) {
									let e = n.devtool;
									const t = h,
										r = h;
									let i = !1;
									const o = h;
									if (!1 !== e) {
										const e = {
											instancePath: s + "/devtool",
											schemaPath: "#/definitions/DevTool/anyOf/0/enum",
											keyword: "enum",
											params: {},
											message: 'must pass "enum" keyword validation'
										};
										null === m ? (m = [e]) : m.push(e), h++;
									}
									var f = o === h;
									if (((i = i || f), !i)) {
										const t = h;
										if (h === t)
											if ("string" == typeof e) {
												if (!a.test(e)) {
													const e = {
														instancePath: s + "/devtool",
														schemaPath: "#/definitions/DevTool/anyOf/1/pattern",
														keyword: "pattern",
														params: {
															pattern:
																"^(inline-|hidden-|eval-)?(nosources-)?(cheap-(module-)?)?source-map$"
														},
														message:
															'must match pattern "^(inline-|hidden-|eval-)?(nosources-)?(cheap-(module-)?)?source-map$"'
													};
													null === m ? (m = [e]) : m.push(e), h++;
												}
											} else {
												const e = {
													instancePath: s + "/devtool",
													schemaPath: "#/definitions/DevTool/anyOf/1/type",
													keyword: "type",
													params: { type: "string" },
													message: "must be string"
												};
												null === m ? (m = [e]) : m.push(e), h++;
											}
										(f = t === h), (i = i || f);
									}
									if (!i) {
										const e = {
											instancePath: s + "/devtool",
											schemaPath: "#/definitions/DevTool/anyOf",
											keyword: "anyOf",
											params: {},
											message: "must match a schema in anyOf"
										};
										return (
											null === m ? (m = [e]) : m.push(e),
											h++,
											(V.errors = m),
											!1
										);
									}
									(h = r),
										null !== m && (r ? (m.length = r) : (m = null)),
										(u = t === h);
								} else u = !0;
								if (u) {
									if (void 0 !== n.entry) {
										const e = h;
										o(n.entry, {
											instancePath: s + "/entry",
											parentData: n,
											parentDataProperty: "entry",
											rootData: l
										}) ||
											((m = null === m ? o.errors : m.concat(o.errors)),
											(h = m.length)),
											(u = e === h);
									} else u = !0;
									if (u) {
										if (void 0 !== n.experiments) {
											let e = n.experiments;
											const t = h;
											if (h == h) {
												if (!e || "object" != typeof e || Array.isArray(e))
													return (
														(V.errors = [
															{
																instancePath: s + "/experiments",
																schemaPath: "#/definitions/Experiments/type",
																keyword: "type",
																params: { type: "object" },
																message: "must be object"
															}
														]),
														!1
													);
												{
													const t = h;
													for (const t in e)
														if (
															"incrementalRebuild" !== t &&
															"lazyCompilation" !== t
														)
															return (
																(V.errors = [
																	{
																		instancePath: s + "/experiments",
																		schemaPath:
																			"#/definitions/Experiments/additionalProperties",
																		keyword: "additionalProperties",
																		params: { additionalProperty: t },
																		message:
																			"must NOT have additional properties"
																	}
																]),
																!1
															);
													if (t === h) {
														if (void 0 !== e.incrementalRebuild) {
															const t = h;
															if ("boolean" != typeof e.incrementalRebuild)
																return (
																	(V.errors = [
																		{
																			instancePath:
																				s + "/experiments/incrementalRebuild",
																			schemaPath:
																				"#/definitions/Experiments/properties/incrementalRebuild/type",
																			keyword: "type",
																			params: { type: "boolean" },
																			message: "must be boolean"
																		}
																	]),
																	!1
																);
															var y = t === h;
														} else y = !0;
														if (y)
															if (void 0 !== e.lazyCompilation) {
																const t = h,
																	a = h;
																let n = !1;
																const r = h;
																if ("boolean" != typeof e.lazyCompilation) {
																	const e = {
																		instancePath:
																			s + "/experiments/lazyCompilation",
																		schemaPath:
																			"#/definitions/Experiments/properties/lazyCompilation/anyOf/0/type",
																		keyword: "type",
																		params: { type: "boolean" },
																		message: "must be boolean"
																	};
																	null === m ? (m = [e]) : m.push(e), h++;
																}
																if (((n = n || r === h), !n)) {
																	const e = {
																		instancePath:
																			s + "/experiments/lazyCompilation",
																		schemaPath:
																			"#/definitions/Experiments/properties/lazyCompilation/anyOf",
																		keyword: "anyOf",
																		params: {},
																		message: "must match a schema in anyOf"
																	};
																	return (
																		null === m ? (m = [e]) : m.push(e),
																		h++,
																		(V.errors = m),
																		!1
																	);
																}
																(h = a),
																	null !== m &&
																		(a ? (m.length = a) : (m = null)),
																	(y = t === h);
															} else y = !0;
													}
												}
											}
											u = t === h;
										} else u = !0;
										if (u) {
											if (void 0 !== n.externals) {
												const e = h;
												p(n.externals, {
													instancePath: s + "/externals",
													parentData: n,
													parentDataProperty: "externals",
													rootData: l
												}) ||
													((m = null === m ? p.errors : m.concat(p.errors)),
													(h = m.length)),
													(u = e === h);
											} else u = !0;
											if (u) {
												if (void 0 !== n.externalsType) {
													let e = n.externalsType;
													const t = h;
													if ("window" !== e && "node-commonjs" !== e)
														return (
															(V.errors = [
																{
																	instancePath: s + "/externalsType",
																	schemaPath:
																		"#/definitions/ExternalsType/enum",
																	keyword: "enum",
																	params: {},
																	message: 'must pass "enum" keyword validation'
																}
															]),
															!1
														);
													u = t === h;
												} else u = !0;
												if (u) {
													if (void 0 !== n.infrastructureLogging) {
														const e = h;
														c(n.infrastructureLogging, {
															instancePath: s + "/infrastructureLogging",
															parentData: n,
															parentDataProperty: "infrastructureLogging",
															rootData: l
														}) ||
															((m = null === m ? c.errors : m.concat(c.errors)),
															(h = m.length)),
															(u = e === h);
													} else u = !0;
													if (u) {
														if (void 0 !== n.mode) {
															let e = n.mode;
															const t = h;
															if (
																"development" !== e &&
																"production" !== e &&
																"none" !== e
															)
																return (
																	(V.errors = [
																		{
																			instancePath: s + "/mode",
																			schemaPath: "#/definitions/Mode/enum",
																			keyword: "enum",
																			params: {},
																			message:
																				'must pass "enum" keyword validation'
																		}
																	]),
																	!1
																);
															u = t === h;
														} else u = !0;
														if (u) {
															if (void 0 !== n.module) {
																const e = h;
																C(n.module, {
																	instancePath: s + "/module",
																	parentData: n,
																	parentDataProperty: "module",
																	rootData: l
																}) ||
																	((m =
																		null === m ? C.errors : m.concat(C.errors)),
																	(h = m.length)),
																	(u = e === h);
															} else u = !0;
															if (u) {
																if (void 0 !== n.name) {
																	const e = h;
																	if ("string" != typeof n.name)
																		return (
																			(V.errors = [
																				{
																					instancePath: s + "/name",
																					schemaPath: "#/definitions/Name/type",
																					keyword: "type",
																					params: { type: "string" },
																					message: "must be string"
																				}
																			]),
																			!1
																		);
																	u = e === h;
																} else u = !0;
																if (u) {
																	if (void 0 !== n.node) {
																		const e = h;
																		L(n.node, {
																			instancePath: s + "/node",
																			parentData: n,
																			parentDataProperty: "node",
																			rootData: l
																		}) ||
																			((m =
																				null === m
																					? L.errors
																					: m.concat(L.errors)),
																			(h = m.length)),
																			(u = e === h);
																	} else u = !0;
																	if (u) {
																		if (void 0 !== n.optimization) {
																			const e = h;
																			S(n.optimization, {
																				instancePath: s + "/optimization",
																				parentData: n,
																				parentDataProperty: "optimization",
																				rootData: l
																			}) ||
																				((m =
																					null === m
																						? S.errors
																						: m.concat(S.errors)),
																				(h = m.length)),
																				(u = e === h);
																		} else u = !0;
																		if (u) {
																			if (void 0 !== n.output) {
																				const e = h;
																				M(n.output, {
																					instancePath: s + "/output",
																					parentData: n,
																					parentDataProperty: "output",
																					rootData: l
																				}) ||
																					((m =
																						null === m
																							? M.errors
																							: m.concat(M.errors)),
																					(h = m.length)),
																					(u = e === h);
																			} else u = !0;
																			if (u) {
																				if (void 0 !== n.plugins) {
																					const e = h;
																					W(n.plugins, {
																						instancePath: s + "/plugins",
																						parentData: n,
																						parentDataProperty: "plugins",
																						rootData: l
																					}) ||
																						((m =
																							null === m
																								? W.errors
																								: m.concat(W.errors)),
																						(h = m.length)),
																						(u = e === h);
																				} else u = !0;
																				if (u) {
																					if (void 0 !== n.resolve) {
																						const e = h;
																						_(n.resolve, {
																							instancePath: s + "/resolve",
																							parentData: n,
																							parentDataProperty: "resolve",
																							rootData: l
																						}) ||
																							((m =
																								null === m
																									? _.errors
																									: m.concat(_.errors)),
																							(h = m.length)),
																							(u = e === h);
																					} else u = !0;
																					if (u) {
																						if (void 0 !== n.snapshot) {
																							let e = n.snapshot;
																							const t = h;
																							if (h == h) {
																								if (
																									!e ||
																									"object" != typeof e ||
																									Array.isArray(e)
																								)
																									return (
																										(V.errors = [
																											{
																												instancePath:
																													s + "/snapshot",
																												schemaPath:
																													"#/definitions/SnapshotOptions/type",
																												keyword: "type",
																												params: {
																													type: "object"
																												},
																												message:
																													"must be object"
																											}
																										]),
																										!1
																									);
																								{
																									const t = h;
																									for (const t in e)
																										if (
																											"module" !== t &&
																											"resolve" !== t
																										)
																											return (
																												(V.errors = [
																													{
																														instancePath:
																															s + "/snapshot",
																														schemaPath:
																															"#/definitions/SnapshotOptions/additionalProperties",
																														keyword:
																															"additionalProperties",
																														params: {
																															additionalProperty:
																																t
																														},
																														message:
																															"must NOT have additional properties"
																													}
																												]),
																												!1
																											);
																									if (t === h) {
																										if (void 0 !== e.module) {
																											let t = e.module;
																											const a = h;
																											if (h === a) {
																												if (
																													!t ||
																													"object" !=
																														typeof t ||
																													Array.isArray(t)
																												)
																													return (
																														(V.errors = [
																															{
																																instancePath:
																																	s +
																																	"/snapshot/module",
																																schemaPath:
																																	"#/definitions/SnapshotOptions/properties/module/type",
																																keyword: "type",
																																params: {
																																	type: "object"
																																},
																																message:
																																	"must be object"
																															}
																														]),
																														!1
																													);
																												{
																													const e = h;
																													for (const e in t)
																														if (
																															"hash" !== e &&
																															"timestamp" !== e
																														)
																															return (
																																(V.errors = [
																																	{
																																		instancePath:
																																			s +
																																			"/snapshot/module",
																																		schemaPath:
																																			"#/definitions/SnapshotOptions/properties/module/additionalProperties",
																																		keyword:
																																			"additionalProperties",
																																		params: {
																																			additionalProperty:
																																				e
																																		},
																																		message:
																																			"must NOT have additional properties"
																																	}
																																]),
																																!1
																															);
																													if (e === h) {
																														if (
																															void 0 !== t.hash
																														) {
																															const e = h;
																															if (
																																"boolean" !=
																																typeof t.hash
																															)
																																return (
																																	(V.errors = [
																																		{
																																			instancePath:
																																				s +
																																				"/snapshot/module/hash",
																																			schemaPath:
																																				"#/definitions/SnapshotOptions/properties/module/properties/hash/type",
																																			keyword:
																																				"type",
																																			params: {
																																				type: "boolean"
																																			},
																																			message:
																																				"must be boolean"
																																		}
																																	]),
																																	!1
																																);
																															var d = e === h;
																														} else d = !0;
																														if (d)
																															if (
																																void 0 !==
																																t.timestamp
																															) {
																																const e = h;
																																if (
																																	"boolean" !=
																																	typeof t.timestamp
																																)
																																	return (
																																		(V.errors =
																																			[
																																				{
																																					instancePath:
																																						s +
																																						"/snapshot/module/timestamp",
																																					schemaPath:
																																						"#/definitions/SnapshotOptions/properties/module/properties/timestamp/type",
																																					keyword:
																																						"type",
																																					params:
																																						{
																																							type: "boolean"
																																						},
																																					message:
																																						"must be boolean"
																																				}
																																			]),
																																		!1
																																	);
																																d = e === h;
																															} else d = !0;
																													}
																												}
																											}
																											var g = a === h;
																										} else g = !0;
																										if (g)
																											if (
																												void 0 !== e.resolve
																											) {
																												let t = e.resolve;
																												const a = h;
																												if (h === a) {
																													if (
																														!t ||
																														"object" !=
																															typeof t ||
																														Array.isArray(t)
																													)
																														return (
																															(V.errors = [
																																{
																																	instancePath:
																																		s +
																																		"/snapshot/resolve",
																																	schemaPath:
																																		"#/definitions/SnapshotOptions/properties/resolve/type",
																																	keyword:
																																		"type",
																																	params: {
																																		type: "object"
																																	},
																																	message:
																																		"must be object"
																																}
																															]),
																															!1
																														);
																													{
																														const e = h;
																														for (const e in t)
																															if (
																																"hash" !== e &&
																																"timestamp" !==
																																	e
																															)
																																return (
																																	(V.errors = [
																																		{
																																			instancePath:
																																				s +
																																				"/snapshot/resolve",
																																			schemaPath:
																																				"#/definitions/SnapshotOptions/properties/resolve/additionalProperties",
																																			keyword:
																																				"additionalProperties",
																																			params: {
																																				additionalProperty:
																																					e
																																			},
																																			message:
																																				"must NOT have additional properties"
																																		}
																																	]),
																																	!1
																																);
																														if (e === h) {
																															if (
																																void 0 !==
																																t.hash
																															) {
																																const e = h;
																																if (
																																	"boolean" !=
																																	typeof t.hash
																																)
																																	return (
																																		(V.errors =
																																			[
																																				{
																																					instancePath:
																																						s +
																																						"/snapshot/resolve/hash",
																																					schemaPath:
																																						"#/definitions/SnapshotOptions/properties/resolve/properties/hash/type",
																																					keyword:
																																						"type",
																																					params:
																																						{
																																							type: "boolean"
																																						},
																																					message:
																																						"must be boolean"
																																				}
																																			]),
																																		!1
																																	);
																																var P = e === h;
																															} else P = !0;
																															if (P)
																																if (
																																	void 0 !==
																																	t.timestamp
																																) {
																																	const e = h;
																																	if (
																																		"boolean" !=
																																		typeof t.timestamp
																																	)
																																		return (
																																			(V.errors =
																																				[
																																					{
																																						instancePath:
																																							s +
																																							"/snapshot/resolve/timestamp",
																																						schemaPath:
																																							"#/definitions/SnapshotOptions/properties/resolve/properties/timestamp/type",
																																						keyword:
																																							"type",
																																						params:
																																							{
																																								type: "boolean"
																																							},
																																						message:
																																							"must be boolean"
																																					}
																																				]),
																																			!1
																																		);
																																	P = e === h;
																																} else P = !0;
																														}
																													}
																												}
																												g = a === h;
																											} else g = !0;
																									}
																								}
																							}
																							u = t === h;
																						} else u = !0;
																						if (u) {
																							if (void 0 !== n.stats) {
																								const e = h;
																								H(n.stats, {
																									instancePath: s + "/stats",
																									parentData: n,
																									parentDataProperty: "stats",
																									rootData: l
																								}) ||
																									((m =
																										null === m
																											? H.errors
																											: m.concat(H.errors)),
																									(h = m.length)),
																									(u = e === h);
																							} else u = !0;
																							if (u) {
																								if (void 0 !== n.target) {
																									let e = n.target;
																									const t = h,
																										a = h;
																									let r = !1;
																									const i = h;
																									if (h === i)
																										if (Array.isArray(e))
																											if (e.length < 1) {
																												const e = {
																													instancePath:
																														s + "/target",
																													schemaPath:
																														"#/definitions/Target/anyOf/0/minItems",
																													keyword: "minItems",
																													params: { limit: 1 },
																													message:
																														"must NOT have fewer than 1 items"
																												};
																												null === m
																													? (m = [e])
																													: m.push(e),
																													h++;
																											} else {
																												const t = e.length;
																												for (
																													let a = 0;
																													a < t;
																													a++
																												) {
																													let t = e[a];
																													const n = h;
																													if (h === n)
																														if (
																															"string" ==
																															typeof t
																														) {
																															if (
																																t.length < 1
																															) {
																																const e = {
																																	instancePath:
																																		s +
																																		"/target/" +
																																		a,
																																	schemaPath:
																																		"#/definitions/Target/anyOf/0/items/minLength",
																																	keyword:
																																		"minLength",
																																	params: {},
																																	message:
																																		'must pass "minLength" keyword validation'
																																};
																																null === m
																																	? (m = [e])
																																	: m.push(e),
																																	h++;
																															}
																														} else {
																															const e = {
																																instancePath:
																																	s +
																																	"/target/" +
																																	a,
																																schemaPath:
																																	"#/definitions/Target/anyOf/0/items/type",
																																keyword: "type",
																																params: {
																																	type: "string"
																																},
																																message:
																																	"must be string"
																															};
																															null === m
																																? (m = [e])
																																: m.push(e),
																																h++;
																														}
																													if (n !== h) break;
																												}
																											}
																										else {
																											const e = {
																												instancePath:
																													s + "/target",
																												schemaPath:
																													"#/definitions/Target/anyOf/0/type",
																												keyword: "type",
																												params: {
																													type: "array"
																												},
																												message: "must be array"
																											};
																											null === m
																												? (m = [e])
																												: m.push(e),
																												h++;
																										}
																									var b = i === h;
																									if (((r = r || b), !r)) {
																										const t = h;
																										if (!1 !== e) {
																											const e = {
																												instancePath:
																													s + "/target",
																												schemaPath:
																													"#/definitions/Target/anyOf/1/enum",
																												keyword: "enum",
																												params: {},
																												message:
																													'must pass "enum" keyword validation'
																											};
																											null === m
																												? (m = [e])
																												: m.push(e),
																												h++;
																										}
																										if (
																											((b = t === h),
																											(r = r || b),
																											!r)
																										) {
																											const t = h;
																											if (h === t)
																												if (
																													"string" == typeof e
																												) {
																													if (e.length < 1) {
																														const e = {
																															instancePath:
																																s + "/target",
																															schemaPath:
																																"#/definitions/Target/anyOf/2/minLength",
																															keyword:
																																"minLength",
																															params: {},
																															message:
																																'must pass "minLength" keyword validation'
																														};
																														null === m
																															? (m = [e])
																															: m.push(e),
																															h++;
																													}
																												} else {
																													const e = {
																														instancePath:
																															s + "/target",
																														schemaPath:
																															"#/definitions/Target/anyOf/2/type",
																														keyword: "type",
																														params: {
																															type: "string"
																														},
																														message:
																															"must be string"
																													};
																													null === m
																														? (m = [e])
																														: m.push(e),
																														h++;
																												}
																											(b = t === h),
																												(r = r || b);
																										}
																									}
																									if (!r) {
																										const e = {
																											instancePath:
																												s + "/target",
																											schemaPath:
																												"#/definitions/Target/anyOf",
																											keyword: "anyOf",
																											params: {},
																											message:
																												"must match a schema in anyOf"
																										};
																										return (
																											null === m
																												? (m = [e])
																												: m.push(e),
																											h++,
																											(V.errors = m),
																											!1
																										);
																									}
																									(h = a),
																										null !== m &&
																											(a
																												? (m.length = a)
																												: (m = null)),
																										(u = t === h);
																								} else u = !0;
																								if (u) {
																									if (void 0 !== n.watch) {
																										const e = h;
																										if (
																											"boolean" !=
																											typeof n.watch
																										)
																											return (
																												(V.errors = [
																													{
																														instancePath:
																															s + "/watch",
																														schemaPath:
																															"#/definitions/Watch/type",
																														keyword: "type",
																														params: {
																															type: "boolean"
																														},
																														message:
																															"must be boolean"
																													}
																												]),
																												!1
																											);
																										u = e === h;
																									} else u = !0;
																									if (u) {
																										if (
																											void 0 !== n.watchOptions
																										) {
																											let e = n.watchOptions;
																											const t = h;
																											if (h == h) {
																												if (
																													!e ||
																													"object" !=
																														typeof e ||
																													Array.isArray(e)
																												)
																													return (
																														(V.errors = [
																															{
																																instancePath:
																																	s +
																																	"/watchOptions",
																																schemaPath:
																																	"#/definitions/WatchOptions/type",
																																keyword: "type",
																																params: {
																																	type: "object"
																																},
																																message:
																																	"must be object"
																															}
																														]),
																														!1
																													);
																												{
																													const t = h;
																													for (const t in e)
																														if (
																															"aggregateTimeout" !==
																																t &&
																															"followSymlinks" !==
																																t &&
																															"ignored" !== t &&
																															"poll" !== t &&
																															"stdin" !== t
																														)
																															return (
																																(V.errors = [
																																	{
																																		instancePath:
																																			s +
																																			"/watchOptions",
																																		schemaPath:
																																			"#/definitions/WatchOptions/additionalProperties",
																																		keyword:
																																			"additionalProperties",
																																		params: {
																																			additionalProperty:
																																				t
																																		},
																																		message:
																																			"must NOT have additional properties"
																																	}
																																]),
																																!1
																															);
																													if (t === h) {
																														if (
																															void 0 !==
																															e.aggregateTimeout
																														) {
																															let t =
																																e.aggregateTimeout;
																															const a = h;
																															if (
																																"number" !=
																																	typeof t ||
																																!isFinite(t)
																															)
																																return (
																																	(V.errors = [
																																		{
																																			instancePath:
																																				s +
																																				"/watchOptions/aggregateTimeout",
																																			schemaPath:
																																				"#/definitions/WatchOptions/properties/aggregateTimeout/type",
																																			keyword:
																																				"type",
																																			params: {
																																				type: "number"
																																			},
																																			message:
																																				"must be number"
																																		}
																																	]),
																																	!1
																																);
																															var k = a === h;
																														} else k = !0;
																														if (k) {
																															if (
																																void 0 !==
																																e.followSymlinks
																															) {
																																const t = h;
																																if (
																																	"boolean" !=
																																	typeof e.followSymlinks
																																)
																																	return (
																																		(V.errors =
																																			[
																																				{
																																					instancePath:
																																						s +
																																						"/watchOptions/followSymlinks",
																																					schemaPath:
																																						"#/definitions/WatchOptions/properties/followSymlinks/type",
																																					keyword:
																																						"type",
																																					params:
																																						{
																																							type: "boolean"
																																						},
																																					message:
																																						"must be boolean"
																																				}
																																			]),
																																		!1
																																	);
																																k = t === h;
																															} else k = !0;
																															if (k) {
																																if (
																																	void 0 !==
																																	e.ignored
																																) {
																																	let t =
																																		e.ignored;
																																	const a = h,
																																		n = h;
																																	let r = !1;
																																	const i = h;
																																	if (h === i)
																																		if (
																																			Array.isArray(
																																				t
																																			)
																																		) {
																																			const e =
																																				t.length;
																																			for (
																																				let a = 0;
																																				a < e;
																																				a++
																																			) {
																																				let e =
																																					t[a];
																																				const n =
																																					h;
																																				if (
																																					h ===
																																					n
																																				)
																																					if (
																																						"string" ==
																																						typeof e
																																					) {
																																						if (
																																							e.length <
																																							1
																																						) {
																																							const e =
																																								{
																																									instancePath:
																																										s +
																																										"/watchOptions/ignored/" +
																																										a,
																																									schemaPath:
																																										"#/definitions/WatchOptions/properties/ignored/anyOf/0/items/minLength",
																																									keyword:
																																										"minLength",
																																									params:
																																										{},
																																									message:
																																										'must pass "minLength" keyword validation'
																																								};
																																							null ===
																																							m
																																								? (m =
																																										[
																																											e
																																										])
																																								: m.push(
																																										e
																																								  ),
																																								h++;
																																						}
																																					} else {
																																						const e =
																																							{
																																								instancePath:
																																									s +
																																									"/watchOptions/ignored/" +
																																									a,
																																								schemaPath:
																																									"#/definitions/WatchOptions/properties/ignored/anyOf/0/items/type",
																																								keyword:
																																									"type",
																																								params:
																																									{
																																										type: "string"
																																									},
																																								message:
																																									"must be string"
																																							};
																																						null ===
																																						m
																																							? (m =
																																									[
																																										e
																																									])
																																							: m.push(
																																									e
																																							  ),
																																							h++;
																																					}
																																				if (
																																					n !==
																																					h
																																				)
																																					break;
																																			}
																																		} else {
																																			const e =
																																				{
																																					instancePath:
																																						s +
																																						"/watchOptions/ignored",
																																					schemaPath:
																																						"#/definitions/WatchOptions/properties/ignored/anyOf/0/type",
																																					keyword:
																																						"type",
																																					params:
																																						{
																																							type: "array"
																																						},
																																					message:
																																						"must be array"
																																				};
																																			null === m
																																				? (m = [
																																						e
																																				  ])
																																				: m.push(
																																						e
																																				  ),
																																				h++;
																																		}
																																	var O =
																																		i === h;
																																	if (
																																		((r =
																																			r || O),
																																		!r)
																																	) {
																																		const e = h;
																																		if (
																																			!(
																																				t instanceof
																																				RegExp
																																			)
																																		) {
																																			const e =
																																				{
																																					instancePath:
																																						s +
																																						"/watchOptions/ignored",
																																					schemaPath:
																																						"#/definitions/WatchOptions/properties/ignored/anyOf/1/instanceof",
																																					keyword:
																																						"instanceof",
																																					params:
																																						{},
																																					message:
																																						'must pass "instanceof" keyword validation'
																																				};
																																			null === m
																																				? (m = [
																																						e
																																				  ])
																																				: m.push(
																																						e
																																				  ),
																																				h++;
																																		}
																																		if (
																																			((O =
																																				e ===
																																				h),
																																			(r =
																																				r || O),
																																			!r)
																																		) {
																																			const e =
																																				h;
																																			if (
																																				h === e
																																			)
																																				if (
																																					"string" ==
																																					typeof t
																																				) {
																																					if (
																																						t.length <
																																						1
																																					) {
																																						const e =
																																							{
																																								instancePath:
																																									s +
																																									"/watchOptions/ignored",
																																								schemaPath:
																																									"#/definitions/WatchOptions/properties/ignored/anyOf/2/minLength",
																																								keyword:
																																									"minLength",
																																								params:
																																									{},
																																								message:
																																									'must pass "minLength" keyword validation'
																																							};
																																						null ===
																																						m
																																							? (m =
																																									[
																																										e
																																									])
																																							: m.push(
																																									e
																																							  ),
																																							h++;
																																					}
																																				} else {
																																					const e =
																																						{
																																							instancePath:
																																								s +
																																								"/watchOptions/ignored",
																																							schemaPath:
																																								"#/definitions/WatchOptions/properties/ignored/anyOf/2/type",
																																							keyword:
																																								"type",
																																							params:
																																								{
																																									type: "string"
																																								},
																																							message:
																																								"must be string"
																																						};
																																					null ===
																																					m
																																						? (m =
																																								[
																																									e
																																								])
																																						: m.push(
																																								e
																																						  ),
																																						h++;
																																				}
																																			(O =
																																				e ===
																																				h),
																																				(r =
																																					r ||
																																					O);
																																		}
																																	}
																																	if (!r) {
																																		const e = {
																																			instancePath:
																																				s +
																																				"/watchOptions/ignored",
																																			schemaPath:
																																				"#/definitions/WatchOptions/properties/ignored/anyOf",
																																			keyword:
																																				"anyOf",
																																			params:
																																				{},
																																			message:
																																				"must match a schema in anyOf"
																																		};
																																		return (
																																			null === m
																																				? (m = [
																																						e
																																				  ])
																																				: m.push(
																																						e
																																				  ),
																																			h++,
																																			(V.errors =
																																				m),
																																			!1
																																		);
																																	}
																																	(h = n),
																																		null !==
																																			m &&
																																			(n
																																				? (m.length =
																																						n)
																																				: (m =
																																						null)),
																																		(k =
																																			a === h);
																																} else k = !0;
																																if (k) {
																																	if (
																																		void 0 !==
																																		e.poll
																																	) {
																																		let t =
																																			e.poll;
																																		const a = h,
																																			n = h;
																																		let r = !1;
																																		const i = h;
																																		if (
																																			"number" !=
																																				typeof t ||
																																			!isFinite(
																																				t
																																			)
																																		) {
																																			const e =
																																				{
																																					instancePath:
																																						s +
																																						"/watchOptions/poll",
																																					schemaPath:
																																						"#/definitions/WatchOptions/properties/poll/anyOf/0/type",
																																					keyword:
																																						"type",
																																					params:
																																						{
																																							type: "number"
																																						},
																																					message:
																																						"must be number"
																																				};
																																			null === m
																																				? (m = [
																																						e
																																				  ])
																																				: m.push(
																																						e
																																				  ),
																																				h++;
																																		}
																																		var w =
																																			i === h;
																																		if (
																																			((r =
																																				r || w),
																																			!r)
																																		) {
																																			const e =
																																				h;
																																			if (
																																				"boolean" !=
																																				typeof t
																																			) {
																																				const e =
																																					{
																																						instancePath:
																																							s +
																																							"/watchOptions/poll",
																																						schemaPath:
																																							"#/definitions/WatchOptions/properties/poll/anyOf/1/type",
																																						keyword:
																																							"type",
																																						params:
																																							{
																																								type: "boolean"
																																							},
																																						message:
																																							"must be boolean"
																																					};
																																				null ===
																																				m
																																					? (m =
																																							[
																																								e
																																							])
																																					: m.push(
																																							e
																																					  ),
																																					h++;
																																			}
																																			(w =
																																				e ===
																																				h),
																																				(r =
																																					r ||
																																					w);
																																		}
																																		if (!r) {
																																			const e =
																																				{
																																					instancePath:
																																						s +
																																						"/watchOptions/poll",
																																					schemaPath:
																																						"#/definitions/WatchOptions/properties/poll/anyOf",
																																					keyword:
																																						"anyOf",
																																					params:
																																						{},
																																					message:
																																						"must match a schema in anyOf"
																																				};
																																			return (
																																				null ===
																																				m
																																					? (m =
																																							[
																																								e
																																							])
																																					: m.push(
																																							e
																																					  ),
																																				h++,
																																				(V.errors =
																																					m),
																																				!1
																																			);
																																		}
																																		(h = n),
																																			null !==
																																				m &&
																																				(n
																																					? (m.length =
																																							n)
																																					: (m =
																																							null)),
																																			(k =
																																				a ===
																																				h);
																																	} else k = !0;
																																	if (k)
																																		if (
																																			void 0 !==
																																			e.stdin
																																		) {
																																			const t =
																																				h;
																																			if (
																																				"boolean" !=
																																				typeof e.stdin
																																			)
																																				return (
																																					(V.errors =
																																						[
																																							{
																																								instancePath:
																																									s +
																																									"/watchOptions/stdin",
																																								schemaPath:
																																									"#/definitions/WatchOptions/properties/stdin/type",
																																								keyword:
																																									"type",
																																								params:
																																									{
																																										type: "boolean"
																																									},
																																								message:
																																									"must be boolean"
																																							}
																																						]),
																																					!1
																																				);
																																			k =
																																				t === h;
																																		} else
																																			k = !0;
																																}
																															}
																														}
																													}
																												}
																											}
																											u = t === h;
																										} else u = !0;
																										if (u)
																											if (
																												void 0 !== n.builtins
																											) {
																												let e = n.builtins;
																												const t = h;
																												if (
																													h === t &&
																													(!e ||
																														"object" !=
																															typeof e ||
																														Array.isArray(e))
																												)
																													return (
																														(V.errors = [
																															{
																																instancePath:
																																	s +
																																	"/builtins",
																																schemaPath:
																																	"#/properties/builtins/type",
																																keyword: "type",
																																params: {
																																	type: "object"
																																},
																																message:
																																	"must be object"
																															}
																														]),
																														!1
																													);
																												u = t === h;
																											} else u = !0;
																									}
																								}
																							}
																						}
																					}
																				}
																			}
																		}
																	}
																}
															}
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}
	return (V.errors = m), 0 === h;
}
