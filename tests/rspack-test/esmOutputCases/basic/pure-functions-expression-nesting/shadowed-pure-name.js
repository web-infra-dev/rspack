function pureFn() {
  return 1;
}

function sideEffect() {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("SHADOWED_PURE_PARAM_MARKER");
  return 1;
}

function localSideEffect() {
  (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("SHADOWED_PURE_LOCAL_MARKER");
  return 1;
}

function shadowedPureName(pureFn) {
  return [pureFn()];
}

function shadowedLocalPureName() {
  const pureFn = localSideEffect;
  return [pureFn()];
}

const functionExpressionNameMarker = "SHADOWED_PURE_FN_EXPR_NAME_MARKER";

const shadowedFunctionExpressionName = function pureFn(flag) {
  return flag && [pureFn(false, functionExpressionNameMarker)];
};

shadowedPureName(sideEffect);
shadowedLocalPureName();
shadowedFunctionExpressionName(true);
