pub trait Store<T, Msg> {
    fn update(&mut self, msg: Msg);
    fn selector<'a, F, O>(&'a self, sel: F) -> Box<dyn Fn () -> O + 'a> where F: Fn(&T) -> O, F: 'a;
}

pub struct ImmutableStore<T, Msg> {
    state: T,
    reducer: fn(&T, Msg) -> T
}

impl<T, Msg> ImmutableStore<T, Msg> {
    pub fn new(initial: T, reducer: fn(&T, Msg) -> T) -> Self {
        ImmutableStore {
            state: initial,
            reducer
        }
    }

    pub fn set(&mut self, value: T) {
        self.state = value;
    }
}

impl<T, Msg> Store<T, Msg> for ImmutableStore<T, Msg> {
    fn update(&mut self, msg: Msg) {
        self.state = (self.reducer)(&self.state, msg)
    }

    fn selector<'a, F, O>(&'a self, sel: F) -> Box<dyn Fn () -> O + 'a> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&self.state))
    }
}