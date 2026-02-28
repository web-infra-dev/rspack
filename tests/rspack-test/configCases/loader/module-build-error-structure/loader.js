module.exports = function () {
    const error = new Error("Cannot be used within pages/_document.js");
    error.name = "NextFontError";
    throw error;
}
