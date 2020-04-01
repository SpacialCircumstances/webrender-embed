pub type Selector<'a, O> = Box<dyn Fn () -> O + 'a>;

pub trait Store<T, Msg> {
    fn update(&mut self, msg: Msg);
    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a;
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

    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&self.state))
    }
}

pub struct MutableStore<T, Msg> {
    state: T,
    reducer: fn(&mut T, Msg)
}

impl<T, Msg> MutableStore<T, Msg> {
    pub fn new(initial: T, reducer: fn(&mut T, Msg)) -> Self {
        MutableStore {
            state: initial,
            reducer
        }
    }
}

impl<T, Msg> Store<T, Msg> for MutableStore<T, Msg> {
    fn update(&mut self, msg: Msg) {
        (self.reducer)(&mut self.state, msg);
    }

    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&self.state))
    }
}