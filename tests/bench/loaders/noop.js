class Breakpoint {
    #callback = null;
    #promise = Promise.resolve();
    #resolve = () => { };

    next() {
        if (this.#callback) {
            this.#callback();
            this.#callback = null;
            this.#promise = new Promise(resolve => {
                this.#resolve = resolve;
            });
        }
    }

    pause(callback) {
        this.#callback = callback;
        this.#resolve();
    }

    async paused() {
        return this.#promise;
    }
}

export const breakpoint = new Breakpoint();

export default function noopLoader(source) {
    const callback = this.async();
    breakpoint.pause(() => {
        callback(null, source);
    });
}
