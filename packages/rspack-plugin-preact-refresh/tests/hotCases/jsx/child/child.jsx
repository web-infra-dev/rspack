throw new Error("should not been imported");
---
export function Child() {
  return (<span>has child</span>);
}
---
export function Child() {
  return (<span>child change</span>);
}
