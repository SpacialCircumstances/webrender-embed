pub struct Store<T, Msg> {
    state: T,
    reducer: fn(&T, Msg) -> T
}

impl<T, Msg> Store<T, Msg> {
    pub fn new(initial: T, reducer: fn(&T, Msg) -> T) -> Self {
        Store {
            state: initial,
            reducer
        }
    }

    pub fn update(&mut self, msg: Msg) {
        self.state = (self.reducer)(&self.state, msg)
    }

    pub fn set(&mut self, value: T) {
        self.state = value;
    }

    pub fn selector<'a, F, O>(&'a self, sel: F) -> Box<dyn Fn () -> O + 'a> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&self.state))
    }
}