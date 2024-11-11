import {
	type DIRTY,
	INVALID,
	type IssueData,
	type ParseContext,
	type ParseInput,
	type ParseReturnType,
	type ProcessedCreateParams,
	type RawCreateParams,
	type SyncParseReturnType,
	ZodError,
	type ZodErrorMap,
	ZodFirstPartyTypeKind,
	type ZodIssue,
	ZodIssueCode,
	ZodType,
	type ZodTypeDef,
	ZodUnion,
	type ZodUnionOptions,
	addIssueToContext,
	z
} from "zod";
import type { RspackOptions } from "./types";

/**
 * The following code is modified based on
 * https://github.com/colinhacks/zod/blob/f487d74ecd3ae703ef8932462d14d643e31658b3/src/types.ts
 *
 * MIT Licensed
 * Author Colin McDonnell @colinhacks
 * MIT License
 * https://github.com/colinhacks/zod/blob/main/LICENSE
 */

function processCreateParams(params: RawCreateParams): ProcessedCreateParams {
	if (!params) return {};
	const { errorMap, invalid_type_error, required_error, description } = params;
	if (errorMap && (invalid_type_error || required_error)) {
		throw new Error(
			`Can't use "invalid_type_error" or "required_error" in conjunction with custom error map.`
		);
	}
	if (errorMap) return { errorMap: errorMap, description };
	const customMap: ZodErrorMap = (iss, ctx) => {
		const { message } = params;

		if (iss.code === "invalid_enum_value") {
			return { message: message ?? ctx.defaultError };
		}
		if (typeof ctx.data === "undefined") {
			return { message: message ?? required_error ?? ctx.defaultError };
		}
		if (iss.code !== "invalid_type") return { message: ctx.defaultError };
		return { message: message ?? invalid_type_error ?? ctx.defaultError };
	};
	return { errorMap: customMap, description };
}

/**
 * Modified `z.union` for overriding its `_parse` to support `parent` field of context.
 *
 * We need to use `parent` field to get the root config object.
 */
class RspackZodUnion<T extends ZodUnionOptions> extends z.ZodUnion<T> {
	_parse(input: ParseInput): ParseReturnType<this["_output"]> {
		const { ctx } = this._processInputParams(input);
		const options = this._def.options;

		function handleResults(
			results: { ctx: ParseContext; result: SyncParseReturnType<any> }[]
		) {
			// return first issue-free validation if it exists
			for (const result of results) {
				if (result.result.status === "valid") {
					return result.result;
				}
			}

			for (const result of results) {
				if (result.result.status === "dirty") {
					// add issues from dirty option

					ctx.common.issues.push(...result.ctx.common.issues);
					return result.result;
				}
			}

			// return invalid
			const unionErrors = results.map(
				result => new ZodError(result.ctx.common.issues)
			);

			addIssueToContext(ctx, {
				code: ZodIssueCode.invalid_union,
				unionErrors
			});
			return INVALID;
		}

		if (ctx.common.async) {
			return Promise.all(
				options.map(async option => {
					const childCtx: ParseContext = {
						...ctx,
						common: {
							...ctx.common,
							issues: []
						},
						parent: ctx
					};
					return {
						result: await option._parseAsync({
							data: ctx.data,
							path: ctx.path,
							parent: childCtx
						}),
						ctx: childCtx
					};
				})
			).then(handleResults);
		}
		let dirty: undefined | { result: DIRTY<any>; ctx: ParseContext } =
			undefined;
		const issues: ZodIssue[][] = [];
		for (const option of options) {
			const childCtx: ParseContext = {
				...ctx,
				common: {
					...ctx.common,
					issues: []
				},
				parent: ctx
			};
			const result = option._parseSync({
				data: ctx.data,
				path: ctx.path,
				parent: childCtx
			});

			if (result.status === "valid") {
				return result;
			}

			if (result.status === "dirty" && !dirty) {
				dirty = { result, ctx: childCtx };
			}

			if (childCtx.common.issues.length) {
				issues.push(childCtx.common.issues);
			}
		}

		if (dirty) {
			ctx.common.issues.push(...dirty.ctx.common.issues);
			return dirty.result;
		}

		const unionErrors = issues.map(issues => new ZodError(issues));
		addIssueToContext(ctx, {
			code: ZodIssueCode.invalid_union,
			unionErrors
		});

		return INVALID;
	}

	static create = <T extends ZodUnionOptions>(
		types: T,
		params?: RawCreateParams
	): ZodUnion<T> => {
		return new RspackZodUnion({
			options: types,
			typeName: ZodFirstPartyTypeKind.ZodUnion,
			...processCreateParams(params)
		});
	};
}

ZodUnion.create = RspackZodUnion.create;

export type ZodCrossFieldsOptions = ZodTypeDef & {
	patterns: Array<{
		test: (root: RspackOptions) => boolean;
		type: ZodType;
		issue?: (res: ParseReturnType<any>) => Array<IssueData>;
	}>;
	default: ZodType;
};

export class ZodRspackCrossChecker<T> extends ZodType<T> {
	constructor(private params: ZodCrossFieldsOptions) {
		super(params);
	}
	_parse(input: z.ParseInput): z.ParseReturnType<T> {
		const ctx = this._getOrReturnCtx(input);
		const root = this._getRootData(ctx);

		for (const pattern of this.params.patterns) {
			if (pattern.test(root)) {
				const res = pattern.type._parse(input);
				const issues =
					typeof pattern.issue === "function" ? pattern.issue(res) : [];
				for (const issue of issues) {
					addIssueToContext(ctx, issue);
				}
				return res;
			}
		}
		return this.params.default._parse(input);
	}
	_getRootData(ctx: z.ParseContext) {
		let root = ctx;
		while (root.parent) {
			root = root.parent;
		}
		return root.data;
	}
}
