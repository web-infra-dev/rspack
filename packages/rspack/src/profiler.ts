import { performance } from 'perf_hooks';
import { spawn, ChildProcess } from 'child_process';
import { Session } from 'inspector';
import fs from 'fs';

const profileIndex = process.argv.indexOf('--profile');

function start(): ChildProcess {
    const rspackProcess = spawn('node', ['--cpu-prof', './path-to-rspack/rspack', '-c', 'rspack.config.js'], {
        stdio: 'inherit'
    });

    rspackProcess.on('exit', () => {
        console.log('rspack process has ended');
    });

    return rspackProcess;
}

if (profileIndex > 0) {
    process.argv.splice(profileIndex, 1)
    const next = process.argv[profileIndex]
    if (next && !next.startsWith('-')) {
        process.argv.splice(profileIndex, 1)
    }

    const session = new Session()
    session.connect()

    session.post('Profiler.enable', () => {
        session.post('Profiler.start', () => {
            start();
            session.post('Profiler.stop', (err, { profile }) => {
                if (!err) {
                    fs.writeFileSync(`CPU.${Date.now()}.cpuprofile`, JSON.stringify(profile));
                }
                session.disconnect();
            });
        });
    })
} else {
    start()
}
