class Command {
	constructor(name) {
		this.name = `a:${name}`;
	}
}

function createCommand(name) {
	return new Command(name);
}

exports.createCommand = createCommand;
exports.Command = Command;
