import { z } from "zod";
import type { RuleSetCondition, ResolveOptions } from "../../types";
import { mock } from "../_utils";
import { ruleSetUse } from "./_common/rule-set-use";
import { resolve } from "../resolve";

const baseRuleSetRule = z.strictObject({
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

type RuleSetRule = z.TypeOf<typeof baseRuleSetRule> & {
	oneOf?: RuleSetRule[];
	rules?: RuleSetRule[];
};

function ruleSetRule(): z.ZodType<RuleSetRule> {
	return baseRuleSetRule.extend({
		oneOf: z.lazy(() => ruleSetRule().array()).optional(),
		rules: z.lazy(() => ruleSetRule().array()).optional()
	});
}

function ruleSetCondition() {
	return mock<RuleSetCondition>();
}

export function rules() {
	return z.literal("...").or(ruleSetRule()).array();
}
