import { z } from "zod";
import type { RuleSetCondition, ResolveOptions } from "../../types";
import { mock } from "../_utils";
import { ruleSetUse } from "./_common/rule-set-use";
import { resolve } from "../resolve";

function ruleSetRule() {
	return z.strictObject({
		test: ruleSetCondition().optional(),
		exclude: ruleSetCondition().optional(),
		include: ruleSetCondition().optional(),
		issuer: ruleSetCondition().optional(),
		dependency: ruleSetCondition().optional(),
		resource: ruleSetCondition().optional(),
		resourceFragment: ruleSetCondition().optional(),
		resourceQuery: ruleSetCondition().optional(),
		scheme: ruleSetCondition().optional(),
		mimetype: ruleSetCondition().optional(),
		descriptionData: ruleSetCondition().optional(),
		enforce: z.enum(["pre", "post"]).optional(),
		sideEffects: z.boolean().optional(),
		parser: z.record(z.any()).optional(),
		generator: z.record(z.any()).optional(),
		type: z.string().optional(),
		loader: z.string().optional(),
		options: z.string().or(z.object({})).optional(),
		use: ruleSetUse().optional(),
		resolve: resolve().optional()
	});
}

function ruleSetCondition() {
	return mock<RuleSetCondition>();
}

export function rules() {
	return z.literal("...").or(ruleSetRule()).array();
}
