globalThis.__cssSelfComposesEvaluations =
  (globalThis.__cssSelfComposesEvaluations || 0) + 1;

export const token = `js-token-${globalThis.__cssSelfComposesEvaluations}`;
export const other = `js-other-${globalThis.__cssSelfComposesEvaluations}`;
