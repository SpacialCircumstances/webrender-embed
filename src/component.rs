pub trait Component<DrawCtx, UpdateCtx, Event> {
    fn draw(&self, ctx: &mut DrawCtx);
    fn update(&mut self, ctx: &mut UpdateCtx);
    fn handle_event(&mut self, event: Event);
}