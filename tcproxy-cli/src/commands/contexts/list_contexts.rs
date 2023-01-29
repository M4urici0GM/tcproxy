use tcproxy_core::Command;
use crate::commands::contexts::DirectoryResolver;
use crate::config::{AppConfig, AppContextError};

pub struct ListContextsCommand {
    dir_resolver: Box<dyn DirectoryResolver + 'static>
}

impl ListContextsCommand {
    pub fn new<T>(dir_resolver: T) -> Self where T : DirectoryResolver + 'static {
        Self {
            dir_resolver: Box::new(dir_resolver)
        }
    }
}

impl Command for ListContextsCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let config_path = self.dir_resolver.get_config_file()?;
        let config = AppConfig::load(&config_path)?;

        // TODO: how to test terminal output? 🤔
        let (biggest_name, contexts) = config.contexts()
            .iter()
            .fold((0, vec![]), |(acc, mut lines), (ctx_name, ctx)| {
                let ctx_name = match ctx_name == config.default_context() {
                    true => format!("{} (default)", ctx_name),
                    false => ctx_name.to_owned(),
                };

                let name_len = match acc < ctx_name.len() {
                    true => ctx_name.len(),
                    false => acc,
                };

                lines.push((ctx_name, ctx.host().to_owned()));
                (name_len, lines)
            });


        println!("{0: <width$}  {1: <width$}", "Context Name", "Server Address", width = biggest_name);
        for (ctx_name, host) in contexts {
            println!("{0: <width$}  {1: <width$}", ctx_name, host, width = biggest_name);
        }

        Ok(())
    }
}