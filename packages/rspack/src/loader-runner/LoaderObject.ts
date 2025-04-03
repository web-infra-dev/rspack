import assert from "node:assert";
import type { JsLoaderItem } from "@rspack/binding";

import type { Compiler } from "../Compiler";
import { BUILTIN_LOADER_PREFIX } from "../config/adapterRuleUse";
import { parseResourceWithoutFragment } from "../util/identifier";

function createLoaderObject(
	loader: JsLoaderItem,
	compiler: Compiler
): LoaderObject {
	const obj: any = {
		path: null,
		query: null,
		fragment: null,
		options: null,
		ident: null,
		normal: null,
		pitch: null,
		raw: null,
		data: null,
		pitchExecuted: false,
		normalExecuted: false
	};
	Object.defineProperty(obj, "request", {
		enumerable: true,
		get: () =>
			obj.path.replace(/#/g, "\u200b#") +
			obj.query.replace(/#/g, "\u200b#") +
			obj.fragment,
		set: (value: JsLoaderItem) => {
			const splittedRequest = parseResourceWithoutFragment(value.loader);
			obj.path = splittedRequest.path;
			obj.query = splittedRequest.query;
			obj.fragment = "";
			obj.options =
				obj.options === null
					? splittedRequest.query
						? splittedRequest.query.slice(1)
						: undefined
					: obj.options;

			if (typeof obj.options === "string" && obj.options[0] === "?") {
				const ident = obj.options.slice(1);
				if (ident === "[[missing ident]]") {
					throw new Error(
						"No ident is provided by referenced loader. " +
							"When using a function for Rule.use in config you need to " +
							"provide an 'ident' property for referenced loader options."
					);
				}
				obj.options = compiler.__internal__ruleSet.references.get(ident);
				if (obj.options === undefined) {
					throw new Error("Invalid ident is provided by referenced loader");
				}
				obj.ident = ident;
			}

			// CHANGE: `rspack_core` returns empty string for `undefined` type.
			// Comply to webpack test case: tests/webpack-test/cases/loaders/cjs-loader-type/index.js
			obj.type = value.type === "" ? undefined : value.type;
			if (obj.options === null) obj.query = "";
			else if (obj.options === undefined) obj.query = "";
			else if (typeof obj.options === "string") obj.query = `?${obj.options}`;
			else if (obj.ident) obj.query = `??${obj.ident}`;
			else if (typeof obj.options === "object" && obj.options.ident)
				obj.query = `??${obj.options.ident}`;
			else obj.query = `?${JSON.stringify(obj.options)}`;
		}
	});
	obj.request = loader;
	if (Object.preventExtensions) {
		Object.preventExtensions(obj);
	}
	return obj;
}

export class LoaderObject {
	request: string;
	path: string;
	query: string;
	fragment: string;
	options?: string | object;
	ident: string;
	normal?: Function;
	pitch?: Function;
	raw?: boolean;
	type?: "module" | "commonjs";
	parallel?: boolean;
	/**
	 * @internal This field is rspack internal. Do not edit.
	 */
	loaderItem: JsLoaderItem;

	constructor(loaderItem: JsLoaderItem, compiler: Compiler) {
		const {
			request,
			path,
			query,
			fragment,
			options,
			ident,
			normal,
			pitch,
			raw,
			type
		} = createLoaderObject(loaderItem, compiler);
		this.request = request;
		this.path = path;
		this.query = query;
		this.fragment = fragment;
		this.options = options;
		this.ident = ident;
		this.normal = normal;
		this.pitch = pitch;
		this.raw = raw;
		this.type = type;
		this.parallel = ident
			? compiler.__internal__ruleSet.references.get(`${ident}$$parallelism`)
			: false;
		this.loaderItem = loaderItem;
		this.loaderItem.data = this.loaderItem.data ?? {};
	}

	get pitchExecuted() {
		return this.loaderItem.pitchExecuted;
	}

	set pitchExecuted(value: boolean) {
		assert(value);
		this.loaderItem.pitchExecuted = true;
	}

	get normalExecuted() {
		return this.loaderItem.normalExecuted;
	}

	set normalExecuted(value: boolean) {
		assert(value);
		this.loaderItem.normalExecuted = true;
	}

	shouldYield() {
		return this.request.startsWith(BUILTIN_LOADER_PREFIX);
	}

	static __from_binding(
		loaderItem: JsLoaderItem,
		compiler: Compiler
	): LoaderObject {
		return new this(loaderItem, compiler);
	}

	static __to_binding(loader: LoaderObject): JsLoaderItem {
		return loader.loaderItem;
	}
}
