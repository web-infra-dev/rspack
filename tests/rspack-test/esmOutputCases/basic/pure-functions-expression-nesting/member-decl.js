export function memberPure(value) {
  return value;
}

export const optionalMemberObj = {
  get value() {
    (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("UNSAFE_OPTIONAL_MEMBER_MARKER");
    return 1;
  },
};
