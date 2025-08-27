// Directly used exports
export const used = 42;
export const alsoUsed = "directly imported";

// Unused exports (should be tree-shaken)
export const unused = 1;
export const alsoUnused = "not imported anywhere";
export const neverUsed = { value: "tree-shaken" };

// Externally preserved exports (not directly used but kept via external-usage.json)
export const externallyUsed1 = "preserved for remote-app";
export const externallyUsed2 = "preserved for analytics";
export const sharedUtility = () => "external system needs this";

// Mixed case - used locally AND externally marked
export const usedBoth = "used locally and externally";
