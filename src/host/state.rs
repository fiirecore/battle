#[derive(Default)]
pub struct StateInstance<S> {
    pub current: FullState<S>,
    next: Option<S>,
}

impl<S> StateInstance<S> {
    pub fn cycle(&mut self) {
        match self.current.1 {
            Substate::Begin => self.current.1 = Substate::Update,
            Substate::Update => if self.next.is_some() {
                self.current.1 = Substate::End;
            },
            Substate::End => if let Some(next) = self.next.take() {
                self.current = FullState(next, Substate::default());
            },
        }
    }

    pub fn set(&mut self, next: S) {
        self.next = Some(next);
    }
}

#[derive(Default)]
pub struct FullState<S>(pub S, pub Substate);

#[derive(Default, Clone, Copy)]
pub enum Substate {
    #[default]
    Begin,
    Update,
    End,
}
