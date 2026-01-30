use iced::advanced::Widget;
use iced::advanced::renderer::Quad;

pub struct TimeLine <Message>{
    length: usize,
    pointer: usize,
    start:usize,
    end:usize,
    click_message:Option<fn(usize)->Message>,
    state:TimeLineState
}

impl<Message> TimeLine<Message>{
    pub fn new(length:usize, pointer:usize, start:usize, end:usize, click_message:Option<fn(usize)->Message>)->Self{
        Self{length, pointer, start, end, click_message, state:Default::default()}
    }

    pub fn x_position(&self, pos:usize, bounds:iced::Rectangle)->f32{
        bounds.x+bounds.width*(pos as f32)/(self.length as f32)
    }
}

impl<Message,Theme, Renderer> Widget<Message,Theme, Renderer> for TimeLine<Message>
where
    Renderer: iced::advanced::Renderer,
{

    // iced-rs is rapidly developed. Wow.
    // fn width(&self) -> iced::Length {
    //     iced::Length::Fill
    // }
    //
    // fn height(&self) -> iced::Length {
    //     iced::Length::Shrink
    // }


    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size { width: iced::Length::Fill, height: iced::Length::Shrink }
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(TimeLineState::default())
    }

    fn layout(
        &mut self,
        _tree:&mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // let mut bounds = limits.();
        // //bounds.height = 100.0;
        // iced::advanced::layout::Node::new(bounds)
        iced::advanced::layout::atomic(limits, iced::Length::Fill, iced::Length::Fill)
    }

    fn draw(
        &self,
        _state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();

        let borderstyle = iced::Border { color: iced::Color::BLACK, width: 1.0 , radius: 0.0.into() };
        let no_borderstyle = iced::Border { color: iced::Color::BLACK, width: 0.0 , radius: 0.0.into() };
        //renderer.fill_quad(Quad{bounds, border_radius:0.0.into(), border_width:1.0, border_color:iced::Color::BLACK}, iced::Color::WHITE);
        renderer.fill_quad(Quad {
            bounds, border: borderstyle,
            // shadow: Default::default()
            ..Default::default()
        }, iced::Color::WHITE);

        let x1 = self.x_position(self.start, bounds);
        let x2 = self.x_position(self.end, bounds);

        // renderer.fill_quad(Quad {
        //     bounds: iced::Rectangle { x: x1, y: bounds.y, width: x2-x1, height: bounds.height },
        //     border_radius: 0.0.into(),
        //     border_width: 1.0,
        //     border_color: iced::Color::BLACK
        //
        // }, iced::Color::new(0.7, 0.7, 0.7, 1.0));
        renderer.fill_quad(Quad {
                           bounds: iced::Rectangle { x: x1, y: bounds.y, width: x2-x1, height: bounds.height },
                           border: borderstyle,
                           ..Default::default()}, iced::Color::from_rgba(0.7, 0.7, 0.7, 1.0));

        let pointer_pos = self.x_position(self.pointer, bounds);
        // renderer.fill_quad(Quad {
        //     bounds: iced::Rectangle { x: pointer_pos-1.0, y: bounds.y, width: 2.0, height: bounds.height },
        //     border_radius: 0.0.into(),
        //     border_width: 0.0,
        //     border_color: iced::Color::WHITE
        //
        // }, iced::Color::new(1.0, 0.0, 0.0, 1.0));

        renderer.fill_quad(Quad {
                           bounds: iced::Rectangle { x: pointer_pos-1.0, y: bounds.y, width: 2.0, height: bounds.height },
                           border: no_borderstyle,
                           ..Default::default() }, iced::Color::from_rgba(1.0, 0.0, 0.0, 1.0));

        if let Some(pos) = cursor.position(){
            if bounds.contains(pos){
                let pos = pos - bounds.position();
                let new_x:usize = (pos.x/bounds.width*(self.length as f32)).round() as usize;
                let cur_x = self.x_position(new_x, bounds);
                // renderer.fill_quad(Quad {
                //     bounds: iced::Rectangle { x: cur_x-1.0, y: bounds.y, width: 2.0, height: bounds.height },
                //     border_radius: 0.0.into(),
                //     border_width: 0.0,
                //     border_color: iced::Color::WHITE
                //
                // }, iced::Color::BLACK);
                renderer.fill_quad(Quad {
                                   bounds: iced::Rectangle { x: cur_x-1.0, y: bounds.y, width: 2.0, height: bounds.height },
                                   border: iced::Border { color: iced::Color::WHITE, width: 0.0, radius: 0.0.into()},
                                   ..Default::default()
                                   }, iced::Color::BLACK);
            }
        }
    }

    fn update(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
            let bounds = layout.bounds();
            let tl_state = &mut self.state;
            if let Some(pos) = cursor.position(){
                if bounds.contains(pos){
                    let pos = pos - bounds.position();
                    match event {
                        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))=>{ tl_state.is_dragging = true;}
                        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left))=>{ tl_state.is_dragging = false;}
                        iced::Event::Mouse(iced::mouse::Event::CursorLeft)=>{tl_state.is_dragging = false;}
                        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position:_ })=>(),
                        _=>{},
                    }
                    if tl_state.is_dragging{
                        let new_x:usize = (pos.x/bounds.width*(self.length as f32)).round() as usize;
                        if let Some(creator) = self.click_message{
                            shell.publish(creator(new_x));
                        }
                    }
                }
                else{
                    tl_state.is_dragging = false;
                }
            }
            // if let Some(pos) = cursor.position(){
            //     let pos = pos - bounds.position();
            //     if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = event{
            //         let new_x:usize = (pos.x/bounds.width*(self.length as f32)) as usize;
            //         if let Some(creator) = self.click_message{
            //             shell.publish(creator(new_x))
            //         }
            //         return iced::event::Status::Captured;
            //     }
            //
            // }
            // iced::event::Status::Ignored
    }

}

#[derive(Default)]
struct TimeLineState{
    is_dragging:bool
}
