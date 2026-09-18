export default {
    findBundle(i, config, step) {
        if (step === "0") {
            return [];
        }
        return ["main.js"];
    }
};
