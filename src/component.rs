pub trait Component<DrawCtx, RenderData, UpdateCtx, Event> {
    fn draw(&self, ctx: &mut DrawCtx, render_data: &RenderData);
    fn update(&mut self, ctx: &mut UpdateCtx);
    fn handle_event(&mut self, event: Event);
}