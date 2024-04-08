use iced::{widget, Font};
use iced_aw::{card, SelectionListStyles, SelectionList};
use super::sparse_intervals::{self, Interval};
use super::messages::{SelectionMessage,TriggerMessage};

pub struct IntervalSelectionDialog{
    unmarked_intervals:sparse_intervals::IntervalStorage,
    intervals_strings:Vec<String>,
    start_entry:String,
    end_entry:String,
    start:usize,
    end:usize,
    manual_select: Option<usize>
}

impl IntervalSelectionDialog{
    pub fn new(unmarked_intervals:sparse_intervals::IntervalStorage)->Self{
        let (start_s,end_s,start,end) = if let Some(v) = unmarked_intervals.get_first_available(){
            (
                v.start.to_string(),
                v.end.to_string(),
                v.start,
                v.end
            )
        }
        else{
            ("".into(),"".into(),0,0)
        };
        let intervals_strings = unmarked_intervals.container.iter().map(|x| format!("{}", x)).collect();
        Self {
            unmarked_intervals,
            intervals_strings,
            start_entry:start_s,
            end_entry:end_s,
            start,end,
            manual_select:None
        }
    }

    pub fn overlay(&self)->iced::Element<'_, TriggerMessage>{
        let mut add_footer = false;

        let selection_list = SelectionList::new_with(
            &self.intervals_strings,
            SelectionMessage::IntervalSelected,
            12.0,
            5.0,
            SelectionListStyles::Default,
            self.manual_select,
            Font::default(),
        );

        let contents = if let Some(_) = self.unmarked_intervals.get_first_available(){
            add_footer = true;
            widget::column![
                widget::row![
                    widget::text_input("start", &self.start_entry).on_input(SelectionMessage::SetStart).on_submit(SelectionMessage::CommitInterval),
                    widget::text_input("end", &self.end_entry).on_input(SelectionMessage::SetEnd).on_submit(SelectionMessage::CommitInterval),
                ],
                selection_list
            ]
        }
        else{
            widget::column![widget::text("No available intervals")]
        };

        let contents:iced::Element<'_,SelectionMessage> = contents.into();

        let mut res_card = card::Card::new(
            widget::text("Select interval"),
            contents.map(TriggerMessage::SelectionMessage),
        );

        if add_footer{
            res_card = res_card.foot(
                widget::container(widget::row![
                    widget::button("OK").on_press(TriggerMessage::ConfirmTrigger).width(100),
                    widget::button("Cancel").on_press(TriggerMessage::CancelChoseTrigger).width(100),
                ]).width(iced::Length::Fill).align_x(iced::alignment::Horizontal::Center)
            );
        };

        res_card = res_card.max_height(250.0).max_width(500.0)
            //.width(Length::Shrink)
            .on_close(TriggerMessage::CancelChoseTrigger);
        res_card.into()

    }

    pub fn update(&mut self, msg:SelectionMessage){
        match msg {
            SelectionMessage::SetStart(v) => {self.start_entry = v},
            SelectionMessage::SetEnd(v) => {self.end_entry = v},
            SelectionMessage::CommitInterval => {
                if let Ok(s) = self.start_entry.parse(){
                    self.start = s;
                }
                else{
                    self.start_entry = self.start.to_string();
                }

                if let Ok(e) = self.end_entry.parse(){
                    self.end = e;
                }
                else{
                    self.end_entry = self.end.to_string();
                }
            },
            SelectionMessage::IntervalSelected(index, _)=>{
                self.manual_select = Some(index);
                let interval = self.unmarked_intervals.container[index];
                self.start = interval.start;
                self.end = interval.end;
                self.start_entry = self.start.to_string();
                self.end_entry = self.end.to_string();
            }
        }
    }

    pub fn get_interval(&self)->Interval{
        Interval::new(self.start, self.end)
    }
}
