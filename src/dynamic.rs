use std::{any::Any, cell::RefCell, collections::HashMap, ops::DerefMut, panic::Location};

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
    fn by_state<S: 'static, T: ReadState<S>>(
        &mut self,
        state: T,
        init: impl Fn() -> Widget,
    ) -> Widget {
        let memo: &mut Box<HashMap<S, Widget>> = self
            .memo_by_state
            .entry(std::panic::Location::caller())
            .or_insert(Box::from(HashMap::<S, Widget>::new()))
            .downcast_mut()
            .unwrap();

        todo!()
    }
}
