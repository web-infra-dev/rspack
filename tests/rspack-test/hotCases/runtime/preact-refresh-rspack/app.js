import { h } from "preact";
import { useState } from "preact/hooks";
const signature = $RefreshSig$();
export function App() {
  signature();
  const [count, setCount] = useState(1);
  return h("button", { onClick: () => setCount(count + 1) }, `before: ${count}`);
}
signature(App, "useState");
$RefreshReg$(App, "App");
---
import { h } from "preact";
import { useState } from "preact/hooks";
const signature = $RefreshSig$();
export function App() {
  signature();
  const [count, setCount] = useState(1);
  return h("button", { onClick: () => setCount(count + 1) }, `after: ${count}`);
}
signature(App, "useState");
$RefreshReg$(App, "App");
