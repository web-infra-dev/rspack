export const angularStyle = resolvedSourceMapUrl => `body {}\n/*# sourceMappingURL=${resolvedSourceMapUrl} */`;
export const multiline = `
//# sourceMappingURL=missing.map
`;
export const nested = `${`//# sourceMappingURL=missing.map`}`;
export const quoted = "/*# sourceMappingURL=missing.map */";
export const escaped = `\`
//# sourceMappingURL=missing.map
`;
export const regex = /[//# sourceMappingURL=missing.map]/;
