module.exports = function (config) {
    // TODO: native watcher cannot correctly handle symlink changes
    return config.experiments?.nativeWatcher === true;
}