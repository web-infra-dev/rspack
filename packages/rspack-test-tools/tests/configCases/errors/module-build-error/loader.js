module.exports = function loader() {
    const error = new Error("Custom mesasge");
    error.name = "CustomError";
    throw error;
}
