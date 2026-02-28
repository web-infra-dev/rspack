export type RuntimeSpec = string | Set<string> | undefined;

export function toJsRuntimeSpec(
  runtime: RuntimeSpec,
): string | string[] | undefined {
  if (runtime instanceof Set) {
    return Array.from(runtime);
  }
  return runtime;
}
