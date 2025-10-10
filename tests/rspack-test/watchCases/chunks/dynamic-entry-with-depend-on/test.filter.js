module.exports = function (config) {
    let nativeWatcherEnabled = config.experiments?.nativeWatcher === true;

    if (process.platform === "win32" && nativeWatcherEnabled) {
        return false;
    }
    return true;
}