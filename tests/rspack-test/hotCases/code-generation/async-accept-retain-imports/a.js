export const value = await Promise.resolve(1);
---
import state from "./state";
state.started.push("a");
await state.pending;
export const value = 2;
