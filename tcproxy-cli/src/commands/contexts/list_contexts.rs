use tcproxy_core::Command;

use crate::config::{AppContextError, Config};

pub struct ListContextsCommand {
    config: Config,
}

impl ListContextsCommand {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl Command for ListContextsCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let context_manager = self.config.lock_context_manager()?;

        // TODO: how to test terminal output? 🤔
        let (biggest_name, contexts) = context_manager.contexts().iter().fold(
            (0, vec![]),
            |(acc, mut lines), (ctx_name, ctx)| {
                let ctx_name = match ctx_name == context_manager.default_context_str() {
                    true => format!("{} (default)", ctx_name),
                    false => ctx_name.to_owned(),
                };

                let name_len = match acc < ctx_name.len() {
                    true => ctx_name.len(),
                    false => acc,
                };

                lines.push((ctx_name, ctx.host().to_owned()));
                (name_len, lines)
            },
        );

        println!(
            "{0: <width$}  {1: <width$}",
            "Context Name",
            "Server Address",
            width = biggest_name
        );
        for (ctx_name, host) in contexts {
            println!(
                "{0: <width$}  {1: <width$}",
                ctx_name,
                host,
                width = biggest_name
            );
        }

        Ok(())
    }
}
