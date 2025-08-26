module.exports = {
    findBundle(i, config, step) {
        if (step === "0") {
            return [];
        }
        return ["main.js"];
    }
};
