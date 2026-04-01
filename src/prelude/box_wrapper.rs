#[derive(Debug)]
pub struct BoxWrapper(pub gtk::Box);

impl BoxWrapper {
    pub fn build(self) -> gtk::Box {
        self.0
    }
}
