use std::{any::Any, collections::HashMap, panic::Location};

use gtk::Widget;

use crate::state::ReadState;

struct WidgetMemo {
    memo: HashMap<&'static Location<'static>, Widget>,
    memo_by_state: HashMap<&'static Location<'static>, Box<dyn Any>>,
}

impl WidgetMemo {
    #[track_caller]
    fn by_location(&mut self, init: impl FnOnce() -> Widget) -> &Widget {
        self.memo
            .entry(std::panic::Location::caller())
            .or_insert(init())
    }

    #[track_caller]
    fn by_state<S: 'static + Clone, T: ReadState<S>>(
        &mut self,
        _state: T,
        _init: impl Fn() -> Widget,
    ) -> Widget {
        let _memo: &mut Box<HashMap<S, Widget>> = self
            .memo_by_state
            .entry(std::panic::Location::caller())
            .or_insert(Box::from(HashMap::<S, Widget>::new()))
            .downcast_mut()
            .unwrap();

        todo!()
    }
}
