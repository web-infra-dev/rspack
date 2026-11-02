export const value = await Promise.resolve("initial");
---
export const value = ;
---
<force_write>export const value = ;
---
export const value = "recovered";
