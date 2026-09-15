new Worker(new URL("./worker-a.js", import.meta.url));
new Worker(new URL("./worker-b.js", import.meta.url));

export const value = "shared";
