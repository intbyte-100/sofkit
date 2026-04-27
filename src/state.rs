use crate::prelude::state_ext::StateHolderExt;
use crate::scheduler::Scheduler;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::{any::Any, rc::Weak};

use gtk::Widget;
use gtk::glib::object::IsA;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::{self, Object};

use crate::batching::BatchGate;

pub trait ReadState<T: 'static>: Clone {
    fn subscribe<W: Fn(&StateAccessor<T>) + 'static>(&self, callback: W) -> Option<Subscription>;

    #[inline]
    fn subscribe_widget<W: Fn(&StateAccessor<T>) + 'static>(
        &self,
        widget: &impl IsA<Widget>,
        callback: W,
    ) -> Option<()> {
        self.subscribe(callback)
            .map(|s| widget.attach_subscription(s))
    }

    fn with<W: FnOnce(&T) -> D, D>(&self, callback: W) -> Option<D>;

    fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.with(|it| it.clone())
    }

    fn map<M: 'static, C>(&self, map: C) -> MappedState<Self, T, M, C>
    where
        C: Fn(&T) -> M + Clone + 'static,
        Self: Sized,
    {
        MappedState::new(self.clone(), map)
    }
}

pub trait WriteState<T: 'static>: Clone {
    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()>;

    fn update(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }
}

pub trait State<T: 'static>: ReadState<T> + WriteState<T> {}

pub struct StateAccessor<T> {
    value: RefCell<T>,
}

impl<T> StateAccessor<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
        }
    }

    pub fn with<W: FnOnce(&T) -> M, M>(&self, callback: W) -> M {
        callback(&self.value.borrow())
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.borrow().clone()
    }

    fn with_mut<W: FnOnce(&mut T) -> M, M>(&self, callback: W) -> M {
        callback(&mut self.value.borrow_mut())
    }
}

pub struct StateCell<T> {
    frame_gate: BatchGate,
    state: StateAccessor<T>,
    subscribers: RefCell<HashMap<usize, Box<dyn Fn(&StateAccessor<T>)>>>,
}

impl<T: 'static> StateCell<T> {
    fn new(state: T) -> Self {
        Self {
            frame_gate: BatchGate::new(),
            state: StateAccessor::new(state),
            subscribers: RefCell::default(),
        }
    }

    #[track_caller]
    fn notify_subscribers_if_needed(self: Rc<Self>) {
        if self.frame_gate.should_run() {
            Scheduler::get().schedule(move || {
                let mut subscribers =
                    std::mem::take::<HashMap<_, _>>(&mut self.subscribers.borrow_mut());

                for subscriber in subscribers.values() {
                    subscriber(&self.state);
                }

                let mut_ref = &mut self.subscribers.borrow_mut();

                std::mem::swap(&mut subscribers, mut_ref);

                mut_ref.extend(subscribers);
            });
        }
    }
}

thread_local! {
    static SUBSCRIPTION_ID: Cell<usize> = Cell::default();
}

fn new_subscription_id() -> usize {
    SUBSCRIPTION_ID.with(|it| {
        let id = it.get();
        it.set(id + 1);
        id
    })
}

#[derive(Clone)]
pub struct StateHandle<T> {
    inner: Weak<StateCell<T>>,
}

impl<T: 'static + Clone> StateHandle<T> {
    fn subscribe<W: Fn(&StateAccessor<T>) + 'static>(&self, callback: W) -> Option<Subscription> {
        let weak = self.inner.clone();
        self.inner.upgrade().map(|it| {
            callback(&it.state);

            let id = new_subscription_id();
            it.subscribers.borrow_mut().insert(id, Box::new(callback));

            Subscription::new(Box::new(move || {
                if let Some(it) = weak.upgrade() {
                    it.subscribers.borrow_mut().remove(&id);
                }
            }))
        })
    }

    #[track_caller]
    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        if let Some(it) = self.inner.upgrade() {
            it.state.with_mut(callback);
            it.notify_subscribers_if_needed();
            Some(())
        } else {
            if cfg!(debug_assertions) {
                eprintln!("Warning: State used without attach_state_holder()");
            }
            None
        }
    }

    pub fn get(&self) -> Option<Rc<StateCell<T>>> {
        self.inner.upgrade()
    }

    #[track_caller]
    pub fn set(&self, value: T) -> Option<()> {
        self.edit(move |it| *it = value)
    }
}

impl<T: 'static + Clone> ReadState<T> for StateHandle<T> {
    fn subscribe<W: Fn(&StateAccessor<T>) + 'static>(&self, callback: W) -> Option<Subscription> {
        StateHandle::subscribe(self, callback)
    }

    fn with<W: FnOnce(&T) -> D, D>(&self, callback: W) -> Option<D> {
        self.inner.upgrade().map(|cell| cell.state.with(callback))
    }
}

impl<T: 'static + Clone> WriteState<T> for StateHandle<T> {
    fn edit<W: FnOnce(&mut T) + 'static>(&self, callback: W) -> Option<()> {
        StateHandle::edit(self, callback)
    }
}

impl<T: 'static + Clone> State<T> for StateHandle<T> {}

struct InnerMappedState<S, F: 'static, M: 'static, C>
where
    S: ReadState<F> + 'static,
    C: Fn(&F) -> M + Clone + 'static,
{
    state: S,
    cached: RefCell<Option<StateAccessor<M>>>,
    gate: BatchGate,
    map: C,
    _marker: std::marker::PhantomData<(F, M)>,
}

impl<S, F: 'static, M: 'static, C> InnerMappedState<S, F, M, C>
where
    S: ReadState<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    fn new(state: S, map: C) -> Rc<Self> {
        Self {
            cached: state.with(map.clone()).map(StateAccessor::new).into(),
            state,
            gate: BatchGate::new(),
            map,
            _marker: std::marker::PhantomData,
        }
        .into()
    }

    fn apply_map(&self, value: &F) {
        if self.gate.should_run() {
            self.cached
                .replace(Some(StateAccessor::new((self.map)(value))));
        }
    }

    fn subscribe<W: Fn(&StateAccessor<M>) + 'static>(
        self: Rc<Self>,
        callback: W,
    ) -> Option<Subscription> {
        let cloned = self.clone();
        self.state.subscribe(move |value| {
            value.with(|it| cloned.apply_map(it));
            callback(cloned.cached.borrow().as_ref().unwrap());
        })
    }

    fn with<W: FnOnce(&M) -> D, D>(self: Rc<Self>, callback: W) -> Option<D> {
        self.cached.borrow().as_ref().map(|it| it.with(callback))
    }
}

#[derive(Clone)]
pub struct MappedState<S, F: 'static, M: 'static, C>
where
    S: ReadState<F> + 'static,
    C: Fn(&F) -> M + Clone + 'static,
{
    inner: Rc<InnerMappedState<S, F, M, C>>,
}

impl<S, F: 'static, M: 'static, C> MappedState<S, F, M, C>
where
    S: ReadState<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    pub fn new(state: S, map: C) -> Self {
        Self {
            inner: InnerMappedState::new(state, map),
        }
    }
}

impl<S, F: 'static + Clone, M: 'static + Clone, C> ReadState<M> for MappedState<S, F, M, C>
where
    S: State<F>,
    C: Fn(&F) -> M + Clone + 'static,
{
    fn subscribe<W: Fn(&StateAccessor<M>) + 'static>(&self, callback: W) -> Option<Subscription> {
        self.inner.clone().subscribe(callback)
    }

    fn with<W: FnOnce(&M) -> D, D>(&self, callback: W) -> Option<D> {
        self.inner.clone().with(callback)
    }
}

mod imp {
    use std::rc::Rc;

    use gtk::glib::subclass::{object::ObjectImpl, types::ObjectSubclass};

    use super::*;

    #[derive(Default)]
    pub struct StateHolder {
        pub states: RefCell<Vec<Rc<dyn Any>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StateHolder {
        const NAME: &'static str = "StateHolder";
        type Type = super::StateHolder;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for StateHolder {}
}

glib::wrapper! {
    pub struct StateHolder(ObjectSubclass<imp::StateHolder>);
}

impl StateHolder {
    pub fn new() -> Self {
        Object::new()
    }

    pub fn state<T: 'static>(&self, value: T) -> StateHandle<T> {
        let rc = Rc::new(StateCell::new(value));
        let weak = Rc::downgrade(&rc);

        self.imp().states.borrow_mut().push(rc);

        StateHandle { inner: weak }
    }
}

pub struct Subscription {
    on_drop: Option<Box<dyn FnOnce()>>,
}

impl Subscription {
    pub fn new(on_drop: Box<dyn FnOnce()>) -> Self {
        Self {
            on_drop: Some(on_drop),
        }
    }

    pub fn unsubscribe(&mut self) {
        if let Some(unsubscribe) = std::mem::take(&mut self.on_drop) {
            glib::idle_add_local_once(unsubscribe);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

pub struct SubscriptionHolder {
    subscription: RefCell<Vec<Subscription>>,
}

impl SubscriptionHolder {
    pub fn new() -> Self {
        Self {
            subscription: Default::default(),
        }
    }

    pub fn attach_subscription(&self, subscription: Subscription) {
        self.subscription.borrow_mut().push(subscription);
    }
}
