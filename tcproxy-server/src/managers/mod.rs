mod connections_manager;
mod port_manager;
mod feature_manager;
mod authentication_manager;
mod account_manager;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
pub use connections_manager::*;
pub use port_manager::*;
pub use feature_manager::*;
pub use authentication_manager::*;
pub use account_manager::*;

pub type IFeatureManager = Box<dyn FeatureManager>;


pub struct Service<T: ?Sized>(Arc<T>);

impl<T> Service<T> {
    pub fn new(service: T) -> Service<T> { Service(Arc::new(service)) }
}

impl<T: ?Sized> Service<T> {
    /// Returns reference to inner `T`.
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    /// Unwraps to the internal `Arc<T>`
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T: ?Sized> From<Arc<T>> for Service<T> {
    fn from(arc: Arc<T>) -> Self {
        Service(arc)
    }
}


pub struct AppContext {
    services: HashMap<TypeId, Box<dyn Any>>
}

fn downcast_owned<T: 'static>(boxed: Box<dyn Any>) -> Option<T> {
    boxed.downcast()
        .ok()
        .map(|boxed| *boxed)
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            services: HashMap::new()
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }

    pub fn register<T: 'static>(&mut self, service: T) -> Option<T> {
        self.services
            .insert(TypeId::of::<T>(), Box::new(service))
            .and_then(downcast_owned)
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.services.contains_key(&TypeId::of::<T>())
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use crate::managers::{AppContext, Service};

    #[test]
    pub fn should_insert_service() {
        struct MyService {

        }

        impl MyService {
            pub fn test(&self) -> u32 {
                32
            }
        }

        let mut ctx = AppContext::new();

        ctx.register(MyService {});

        let recovered_service = ctx.get::<MyService>().unwrap();

        assert_eq!(recovered_service.test(), 32);
    }

    #[test]
    pub fn should_insert_unsized_svc() {
        struct A;
        trait MyService {
            fn test(&self) -> u32 { 32 }
        }

        impl MyService for A {

        }

        let mut context = AppContext::new();
        let dyn_arc: Arc<dyn MyService> = Arc::new(A {});
        let data_arc = Service::from(dyn_arc);

        context.register(data_arc);

        let extracted_service = context.get::<Service<dyn MyService>>().unwrap();

        assert_eq!(A{}.test(), extracted_service.get_ref().test());
    }
}