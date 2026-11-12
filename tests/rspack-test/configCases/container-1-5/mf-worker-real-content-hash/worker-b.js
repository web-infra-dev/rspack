import("remote/value").then(({ default: value }) => self.postMessage(`worker b ${value}`));
