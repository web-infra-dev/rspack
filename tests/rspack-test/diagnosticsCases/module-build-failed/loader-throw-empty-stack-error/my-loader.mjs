export default function () {
    const error = new Error("Failed to load");
    error.stack = "";
    throw error;
};
