export default function (source) {
    this.emitWarning(new Error("Emitted from loader"));
    return source;
}
