use command::Command;

pub mod command;

pub fn kill_player() -> Command {
    Command::kill_command("player")
}

pub fn open_player() -> Command {
    Command::open_command("player")
}
