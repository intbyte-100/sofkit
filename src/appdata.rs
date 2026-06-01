use std::cell::Cell;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::state::{ReadState, StateAccessor, StateHandle, StateHolder};

use std::any::{Any, TypeId};

thread_local! {
    static APPDATA_STACK: RefCell<Vec<Context>> = RefCell::new(Vec::new());
}

#[derive(Default)]
struct Context {
    data: HashMap<TypeId, Rc<dyn Any>>,
}

pub fn appdata() -> AppDataBuilder {
    AppDataBuilder {
        ctx: Context::default(),
    }
}

pub struct AppDataBuilder {
    ctx: Context,
}

impl AppDataBuilder {
    pub fn insert<T: 'static>(mut self, value: T) -> Self {
        self.ctx.data.insert(TypeId::of::<T>(), Rc::new(value));
        self
    }

    pub fn with_data<F, R>(self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        APPDATA_STACK.with(|stack| {
            stack.borrow_mut().push(self.ctx);

            let result = f();

            stack.borrow_mut().pop();

            result
        })
    }
}

pub fn get_appdata<T: 'static>() -> Rc<T> {
    APPDATA_STACK.with(|stack| {
        let stack = stack.borrow();

        for ctx in stack.iter().rev() {
            if let Some(value) = ctx.data.get(&TypeId::of::<T>()) {
                return value
                    .clone()
                    .downcast::<T>()
                    .expect("Type mismatch in AppData");
            }
        }

        panic!("AppData not found for requested type");
    })
}

pub fn has_appdata<T: 'static>() -> bool {
    APPDATA_STACK.with(|stack| {
        let stack = stack.borrow();

        stack
            .iter()
            .rev()
            .any(|ctx| ctx.data.contains_key(&TypeId::of::<T>()))
    })
}

thread_local! {
    static ID: Cell<usize> = Default::default();
}

fn next_id() -> usize {
    ID.with(|id| {
        let result = id.get();
        id.set(result + 1);
        result
    })
}

struct FiledInner<T> {
    data: StateAccessor<T>,
    subscribers: RefCell<HashMap<usize, Box<dyn Fn(Rc<FiledInner<T>>)>>>,
}
pub struct Field<T> {
    inner: Rc<FiledInner<T>>,
}

impl<T> Field<T>
where
    T: Clone + 'static,
{
    pub fn new(data: T) -> Self {
        Self {
            inner: Rc::new(FiledInner {
                data: StateAccessor::new(data),
                subscribers: RefCell::new(HashMap::new()),
            }),
        }
    }

    pub fn set(&self, data: T) {
        self.inner.data.with_mut(|v| *v = data);
        self.notify();
    }

    pub fn edit(&self, f: impl FnOnce(&mut T)) {
        self.inner.data.with_mut(f);
        self.notify();
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.data.with(f)
    }

    fn notify(&self) {
        let mut subscribers = std::mem::take(&mut *self.inner.subscribers.borrow_mut());
        for subscriber in subscribers.values() {
            subscriber(self.inner.clone());
        }

        std::mem::swap(&mut *self.inner.subscribers.borrow_mut(), &mut subscribers);
        self.inner.subscribers.borrow_mut().extend(subscribers);
    }

    fn subscribe(&self, subscriber: impl Fn(Rc<FiledInner<T>>) + 'static) -> FieldSubscription<T> {
        let id = next_id();
        self.inner
            .subscribers
            .borrow_mut()
            .insert(id, Box::new(subscriber));

        FieldSubscription {
            id,
            field: self.inner.clone(),
        }
    }

    pub fn make_state(&self, holder: &StateHolder) -> impl ReadState<T> + 'static{
        let state = holder.state(self.inner.clone());
        let sub_state = state.clone();

        let callback = move |value| {
            sub_state.set(value);
        };

        callback(self.inner.clone());

        let subscription = self.subscribe(callback).into();

        FieldState {
            state,
            subscription,
        }
    }
}

struct FieldSubscription<T> {
    id: usize,
    field: Rc<FiledInner<T>>,
}

impl<T> Drop for FieldSubscription<T> {
    fn drop(&mut self) {
        self.field.subscribers.borrow_mut().remove(&self.id);
    }
}

#[derive(Clone)]
struct FieldState<T>
where
    T: Clone,
{
    state: StateHandle<Rc<FiledInner<T>>>,
    subscription: Rc<FieldSubscription<T>>,
}

impl<T> ReadState<T> for FieldState<T>
where
    T: Clone + 'static,
{
    fn subscribe<W: Fn(&crate::state::StateAccessor<T>) + 'static>(
        &self,
        callback: W,
    ) -> Option<crate::state::Subscription> {
        self.state
            .subscribe(move |value| callback(&value.get().data))
    }

    fn with<W: FnOnce(&T) -> D, D>(&self, callback: W) -> Option<D> {
        self.state.with(|value| value.data.with(callback))
    }
}
