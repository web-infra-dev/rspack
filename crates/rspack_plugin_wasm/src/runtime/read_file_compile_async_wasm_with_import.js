Promise.all([import('fs'), import('url')]).then(([{ readFile }, { URL }]) => new Promise((resolve, reject) => {
    readFile(new URL($PATH, $IMPORT_META_NAME.url), (err, buffer) => {
        if (err) return reject(err);
        // Fake fetch response
        resolve({
            arrayBuffer() { return buffer; }
        })
    });
}));