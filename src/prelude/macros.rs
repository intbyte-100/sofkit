#[macro_export]
macro_rules! hbox {
    ( $( $child:expr ),* $(,)? ) => {{
        use gtk::prelude::*;
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        $(
            let widget = $child.build();
            container.append(&widget);
        )*

        $crate::prelude::box_wrapper::BoxWrapper(container)
    }};
}

#[macro_export]
macro_rules! vbox {
    ( $( $child:expr ),* $(,)? ) => {{
        use gtk::prelude::*;
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);

        $(
            let widget = $child.build();
            container.append(&widget);
        )*

        $crate::prelude::BoxWrapper(container)
    }};
}
