pub trait Component<DrawCtx, Event> {
    fn draw(&self, ctx: &mut DrawCtx);
    fn update(&mut self);
    fn handle_event(&mut self, event: Event);
}