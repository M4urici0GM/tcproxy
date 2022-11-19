use tcproxy_core::Command;

pub struct ListContextsCommand;

impl Command for ListContextsCommand {
    type Output = ();

    fn handle(&mut self) -> Self::Output {
        todo!()
    }
}