use std::collections::HashMap;

use super::{AppContext, AppContextError};

#[derive(Debug, Clone, Default)]
pub struct ContextManager {
    default_context: String,
    user_token: String,
    contexts: Vec<AppContext>,
}

impl ContextManager {
    pub fn new(default_ctx: &str, contexts: &[AppContext]) -> Self {
        Self {
            default_context: String::from(default_ctx),
            contexts: Vec::from(contexts),
            user_token: String::default(),
        }
    }

    pub fn default_context(&self) -> Option<AppContext> {
        self.get_context(&self.default_context)
    }

    pub fn default_context_str(&self) -> &str {
        &self.default_context
    }

    pub fn contexts_arr(&self) -> &[AppContext] {
        &self.contexts
    }

    pub fn set_user_token(&mut self, token: &str) {
        self.user_token = String::from(token);
    }

    pub fn contexts(&self) -> HashMap<String, AppContext> {
        let mut mapped_ctxs = HashMap::new();
        for ctx in self.contexts.iter().cloned() {
            mapped_ctxs.insert(ctx.name().to_owned(), ctx);
        }

        mapped_ctxs
    }

    pub fn ctx_exists(&self, context: &AppContext) -> bool {
        self.contexts.iter().any(|ctx| ctx == context)
    }

    pub fn set_default_context(&mut self, context: &AppContext) -> bool {
        if !self.ctx_exists(context) {
            self.contexts.push(context.clone());
        }

        self.default_context = context.name().to_owned();
        true
    }

    pub fn has_default_context(&self) -> bool {
        self.default_context != String::default()
    }

    pub fn get_default_context(&self) -> Option<AppContext> {
        self.get_context(&self.default_context)
    }

    pub fn get_context(&self, name: &str) -> Option<AppContext> {
        self.contexts
            .iter()
            .cloned()
            .find(|item| item.name() == name)
    }

    pub fn push_context(
        &mut self,
        context: &AppContext,
    ) -> std::result::Result<(), AppContextError> {
        if self.ctx_exists(context) {
            return Err(AppContextError::AlreadyExists(context.clone()));
        }

        self.contexts.push(context.clone());
        if !self.has_default_context() {
            self.set_default_context(context);
        }

        Ok(())
    }
}

