module.exports = function (config) {
    let nativeWatcherEnabled = config.experiments?.nativeWatcher === true;

    // Ref: https://github.com/web-infra-dev/rspack/issues/11828
    if (process.platform === "win32" && nativeWatcherEnabled) {
        return false;
    }
    return true;
}