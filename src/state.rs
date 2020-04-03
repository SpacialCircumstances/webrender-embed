use std::cell::RefCell;

pub type Selector<'a, O> = Box<dyn Fn() -> O + 'a>;

pub trait Store<T, Msg> {
    fn update(&self, msg: Msg);
    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a;
}

pub struct ImmutableStore<T, Msg> {
    state: RefCell<T>,
    reducer: fn(&T, Msg) -> T,
}

impl<T, Msg> ImmutableStore<T, Msg> {
    pub fn new(initial: T, reducer: fn(&T, Msg) -> T) -> Self {
        ImmutableStore {
            state: RefCell::new(initial),
            reducer,
        }
    }

    pub fn set(&self, value: T) {
        let mut state = self.state.borrow_mut();
        *state = value;
    }
}

impl<T, Msg> Store<T, Msg> for ImmutableStore<T, Msg> {
    fn update(&self, msg: Msg) {
        let old_state = self.state.borrow();
        let new_state = (self.reducer)(&*old_state, msg);
        drop(old_state);
        let mut state = self.state.borrow_mut();
        *state = new_state;
    }

    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&*self.state.borrow()))
    }
}

pub struct MutableStore<T, Msg> {
    state: RefCell<T>,
    reducer: fn(&mut T, Msg),
}

impl<T, Msg> MutableStore<T, Msg> {
    pub fn new(initial: T, reducer: fn(&mut T, Msg)) -> Self {
        MutableStore {
            state: RefCell::new(initial),
            reducer,
        }
    }
}

impl<T, Msg> Store<T, Msg> for MutableStore<T, Msg> {
    fn update(&self, msg: Msg) {
        (self.reducer)(&mut *self.state.borrow_mut(), msg);
    }

    fn selector<'a, F, O>(&'a self, sel: F) -> Selector<'a, O> where F: Fn(&T) -> O, F: 'a {
        Box::new(move || sel(&*self.state.borrow()))
    }
}