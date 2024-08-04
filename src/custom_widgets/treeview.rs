use std::{collections::{HashMap, BTreeMap}, cell::RefCell, rc::{Rc, Weak}};
use iced::{advanced::Widget, event};

const STEP:f32 = 22.0;
const HEIGHT:f32 = 20.0;
const X_OFFSET:f32 = 20.0;


#[derive(Debug)]
pub struct TreeNode<T:std::fmt::Debug+Clone>{
    visible:bool,
    name:String,
    pub content:BTreeMap<String,Weak<RefCell<TreeNode<T>>>>,
    pub parent:Weak<RefCell<TreeNode<T>>>,
    last_rect:RefCell<Option<iced::Rectangle>>,
    pub metadata:Option<T>
}

impl<T:std::fmt::Debug+Clone> TreeNode<T>{
    pub fn new(name:String, parent:Weak<RefCell<TreeNode<T>>>, metadata:Option<T>)->Self{
        Self { visible: false, content: BTreeMap::new(), name, parent, last_rect:RefCell::new(None),metadata}
    }

    fn path(&self)->Vec<String>{
        if let Some(parent) = self.parent.upgrade(){
            let mut path = parent.borrow().path();
            path.push(self.name.clone());
            path
        }
        else{
            vec![self.name.clone()]
        }
    }

    pub fn is_final(&self)->bool{
        self.content.is_empty()
    }

    pub fn is_top(&self)->bool{
        self.parent.upgrade().is_none()
    }

    pub fn contains_point(&self, pos: iced::Point)->bool{

        if let Ok(x0) = self.last_rect.try_borrow(){
            if let Some(x) = x0.as_ref(){
                x.contains(pos)
            }
            else{
                false
            }
        }
        else{
            false
        }

    }

    // pub fn get_y_lim(&self)->(f32,f32){
    //     if let Ok(x0) = self.last_rect.try_borrow(){
    //         if let Some(r) = x0.as_ref(){
    //             (r.width+r.x,r.height+r.y)
    //         }
    //         else{
    //             (0.0,0.0)
    //         }
    //     }
    //     else{
    //         (0.0,0.0)
    //     }
    // }

    fn hide(&self){
        *self.last_rect.borrow_mut() = None;
    }

    fn draw<Renderer: iced::advanced::text::Renderer, Theme>(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        x_offset:f32,
        y_offset:f32,
    ) -> f32{
        let prefix = if self.is_final(){
            ""
        }
        else if self.visible{
            "- "
        }
        else{
            "+ "
        };
        let bounds = layout.bounds();
        let label = format!("{}{}",prefix,self.name);
        let x = bounds.x + x_offset;
        let y = y_offset;
        let width = bounds.width - x_offset;
        let height = STEP;
        let new_textrect = iced::Rectangle{x,y,width,height};
        let mut textrect = self.last_rect.borrow_mut();
        *textrect = Some(new_textrect);
        renderer.fill_text(iced::advanced::Text{
            content:&label,
            bounds:bounds.size(),
            size:20.into(),
            line_height: iced::advanced::text::LineHeight::Absolute(iced::Pixels(HEIGHT)),
            //color: iced::Color::BLACK,
            font: renderer.default_font(),
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Top,
            shaping: Default::default(),

        }, iced::Point { x, y},iced::Color::BLACK,new_textrect);

        let mut y_last = y+STEP;
        for (_,node) in self.content.iter(){

            if let Some(node_rc) = node.upgrade(){
                let node_ref = node_rc.borrow();
                if self.visible{
                    y_last = node_ref.draw(state, renderer, theme, style, layout, cursor, viewport, x_offset+X_OFFSET, y_last);
                }
                else {
                    node_ref.hide();
                }
            }
        }

//         if self.visible{
//             for (_,node) in self.content.iter(){
//
//                 if let Some(node_rc) = node.upgrade(){
//                     let node_ref = node_rc.borrow();
//                     y_last = node_ref.draw(state, renderer, theme, style, layout, cursor, viewport, x_offset+X_OFFSET, y_last);
//                 }
//             }
//         }
//         else {
//
//         }
        y_last
        //println!("{}",label);
    }
}

#[derive(Debug)]
pub struct Tree<T:std::fmt::Debug+Clone>{
    nodes:Vec<Rc<RefCell<TreeNode<T>>>>,
    last_height:RefCell<f32>
}

fn new_node<T:std::fmt::Debug+Clone>(name:&str, parent:Weak<RefCell<TreeNode<T>>>,metadata:Option<T>)->Rc<RefCell<TreeNode<T>>>{
    Rc::new(RefCell::new(TreeNode::new(name.to_string(), parent, metadata)))
}

impl<T:std::fmt::Debug+Clone> Tree<T>{
    pub fn new()->Self{
        Self { nodes: Vec::new(), last_height:RefCell::new(1.0)}
    }



    fn insert_node(&mut self, name:&str, parent:Weak<RefCell<TreeNode<T>>>,metadata:Option<T>)->Rc<RefCell<TreeNode<T>>>{
        let newnode = Rc::new(RefCell::new(TreeNode::new(name.to_string(),parent,metadata)));
        // if self.nodes.insert(key.to_string(),newnode.clone()).is_some(){
        //     panic!("overwritten a node!");
        // };
        self.nodes.push(newnode.clone());


        newnode
    }

    pub fn sort_nodes(&mut self){
        for i in 0..self.nodes.len(){
            for j in (1..i).rev(){
                let needs_swap = {self.nodes[j].borrow().name<self.nodes[j-1].borrow().name};
                if needs_swap{
                    self.nodes.swap(j-1, j);
                }
            }
        }
    }

    fn parse_path_in(&mut self, node:Rc<RefCell<TreeNode<T>>>, mut path:Vec<&str>, metadata:Option<T>){
        let mut node_mut = node.borrow_mut();
        let mut splitter = path.drain(..);
        if let Some(category) = splitter.next(){
            let entry = node_mut.content.entry(category.to_string()).or_insert_with(|| Rc::downgrade(&self.insert_node(category,Rc::downgrade(&node),metadata.clone())));
            let rest:Vec<&str> = splitter.collect();
            if let Some(next_node) = Weak::upgrade(entry){
                self.parse_path_in(next_node,rest,metadata);
            }
            else {
                panic!("Invalid plot detected. Investigate this problem");
            }
        }
    }

    fn find_root_with_name(&self, name:&str)->Option<Rc<RefCell<TreeNode<T>>>>{
        for node in self.nodes.iter(){
            if node.borrow().name==name && node.borrow().is_top(){
                return Some(node.clone());
            }
        }
        None
    }

    pub fn parse_path(&mut self, mut path:Vec<&str>, metadata:Option<T>){
        //println!("Parse path {}", path);
        let mut splitter = path.drain(..);
        if let Some(category) = splitter.next(){
            let found = self.find_root_with_name(category);
            let entry = if let Some(v) = found{
                v
            }
            else{
                self.insert_node(category, Weak::new(), metadata.clone())
            };
            let entry = entry.clone();
            let rest = splitter.collect();
            self.parse_path_in(entry,rest,metadata);
        }
    }

    pub fn view<Message,F:Fn(T)->Message>(&self, action:Option<F>)->TreeView<Message,T,F>{
        TreeView::new(self, action)
    }

    fn draw<Theme,Renderer: iced::advanced::text::Renderer>(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ){
        let bounds = layout.bounds();
        let mut y:f32 = bounds.y;
        for node in self.nodes.iter(){
            let node_ref = node.borrow();
            if node_ref.is_top(){
                y = node_ref.draw(state, renderer, theme, style, layout, cursor, viewport, 0.0, y);
            }
        }
        //println!("Last height commit: {}", y-bounds.y);
        *self.last_height.borrow_mut() =  y-bounds.y;
    }

    fn get_last_height(&self)->f32{
        *self.last_height.borrow()
    }
}


pub struct TreeView<'a, Message,T:std::fmt::Debug+Clone,F>
where
    F:Fn(T)->Message
{
    tree:&'a Tree<T>,
    action:Option<F>
}

impl<'a, Message,T:std::fmt::Debug+Clone,F> TreeView<'a, Message, T,F>
where
    F:Fn(T)->Message
{
    pub fn new(tree:&'a Tree<T>, action:Option<F>)->Self{
        Self{tree, action}
    }
}

impl<'a,Message,Theme,Renderer,T:std::fmt::Debug+Clone,F:Fn(T)->Message> Widget<Message, Theme, Renderer> for TreeView<'a, Message,T,F>
where
    Renderer: iced::advanced::text::Renderer
{

    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size { width: iced::Length::Shrink, height: iced::Length::Shrink }
    }

    fn layout(
        &self,
        _tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        // let mut bound = limits.fill();
        // bound.height = self.tree.get_last_height();
        // //bound.width -=10.0;
        // //println!("Last height: {}", bound.height);
        // iced::advanced::layout::Node::new(bound)
        iced::advanced::layout::atomic(limits, iced::Length::Fill, self.tree.get_last_height())
    }

    fn draw(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.tree.draw(state,renderer,theme,style,layout,cursor,viewport);
        // *self.last_height.borrow_mut() = y;
    }

    fn on_event(
        &mut self,
        _state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        _layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced::event::Status {
        if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) = event{
            for node in self.tree.nodes.iter(){
                if let Some(pos) = cursor.position(){
                    let mut node_ref = node.borrow_mut();
                    if node_ref.contains_point(pos){
                        if node_ref.is_final(){
                            if let Some(action) = &self.action{
                                println!("{:?}",node_ref.metadata);
                                if let Some(m) = node_ref.metadata.clone(){
                                    shell.publish(action(m));
                                }
                            }
                            //println!("{}",node_ref.path());
                        }
                        else{
                            node_ref.visible = !node_ref.visible;
                        }
                        return iced::event::Status::Captured;
                    }
                }
            }

        }
        iced::event::Status::Ignored
    }
}
