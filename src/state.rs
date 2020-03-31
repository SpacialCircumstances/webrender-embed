pub struct Store<T> {
    state: T
}

impl<T> Store<T> {
    pub fn new(initial: T) -> Self {
        Store {
            state: initial
        }
    }

    pub fn set(&mut self, value: T) {
        self.state = value;
    }

    pub fn selector<'a, F, O>(&'a self, sel: F) -> Box<dyn Fn () -> O + 'a> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&self.state))
    }
}