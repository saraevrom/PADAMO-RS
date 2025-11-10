use padamo_api::lazy_array_operations::ArrayND;
use padamo_iced_forms::{make_action, IcedForm, IcedFormBuffer, Action};
use serde::{Serialize, Deserialize};

#[derive(Clone,Copy,Debug,Default,IcedForm, Serialize, Deserialize)]
pub enum Selector{
    #[field_name("Maximum")] #[default] Max,
    #[field_name("Mean")] Mean,
    // #[field_name("Median")] Median,
}

impl Selector{
    pub fn apply<T:Iterator<Item = f64>>(&self, input:T)->f64{
        match self{
            Self::Max=>input.max_by(|a, b| a.total_cmp(b)).unwrap_or(0.0),
            Self::Mean=>{
                let mut amount:usize = 0;
                let mut sum = 0.0;
                for i in input{
                    amount += 1;
                    sum += i;
                }
                if amount>0{sum/(amount as f64)}else{0.0}
            }
        }
    }
}

#[derive(Clone,Copy,Debug,Default,IcedForm, Serialize, Deserialize)]
pub enum SelectSign{
    #[field_name("Greater")] #[default] Greater,
    #[field_name("Less")] Less,
}

impl SelectSign{
    pub fn compare(&self, a:f64, b:f64)->bool{
        match self {
            Self::Greater => a>b,
            Self::Less => a<b,
        }
    }
}

#[derive(Clone,Copy,Debug,Default,IcedForm, Serialize, Deserialize)]
pub enum DetectorSelection{
    #[field_name("Primary")] #[default] Primary,
    #[field_name("Secondary")] Secondary,
}

#[derive(Clone,Copy,Debug,Default,IcedForm, Serialize, Deserialize)]
pub enum SelectionMode{
    #[field_name("Add (OR)")] #[default] Add,
    #[field_name("Intersect (AND)")] Intersect,
    #[field_name("Remove (AND NOT)")] Remove,
    #[field_name("Overwrite")] Overwrite,
}

impl SelectionMode{
    pub fn apply(&self, a:bool, b:bool)->bool{
        match self {
            Self::Add => a || b,
            Self::Intersect => a && b,
            Self::Remove => a && !b,
            Self::Overwrite => b,
        }
    }
}

#[derive(Clone, Copy, Debug,Default)]
pub enum SelectorActions{
    #[default] Select
}

make_action!(SelectorSelect, SelectorActions, Select);

#[derive(Clone,Debug,IcedForm, Serialize, Deserialize)]
#[spoiler_hidden]
pub struct SelectForm{
    #[field_name("Select")] _select: Action<SelectorActions, SelectorSelect>,
    #[field_name("where")] selector: Selector,
    #[field_name("is")] sign: SelectSign,
    #[field_name("than")] threshold: f64,
    #[field_name("in detector")] pub detector: DetectorSelection,
    #[field_name("using mode")] mode: SelectionMode,
}

impl Default for SelectForm{
    fn default() -> Self {
        Self{
            _select: Default::default(),
            threshold: 1.5,
            selector: Selector::Max,
            sign: SelectSign::Greater,
            detector: DetectorSelection::Primary,
            mode: SelectionMode::Add,
        }
    }
}

impl SelectForm{
    fn select_data(&self, data:&ArrayND<f64>)->ArrayND<bool>{
        let shape = data.shape.clone();
        if shape.len()<=1{
            panic!("Called selector on bad data");
        }

        let iterators = data.make_pixel_iterators();
        let res_shape = iterators.shape;
        let values:Vec<bool> = iterators.flat_data.into_iter()
            .map(|x| self.selector.apply(x))  // Check values
            .map(|x| self.sign.compare(x, self.threshold))  // Comparison
            .collect();
        ArrayND { flat_data: values.into(), shape: res_shape }
    }

    pub fn modify_mask(&self, mask:&mut ArrayND<bool>, data: &ArrayND<f64>){
        let modifier = self.select_data(data);
        mask.flat_data.iter_mut().zip(modifier.flat_data.iter()).for_each(|(a,b)|{
            *a = self.mode.apply(*a, *b);
        });
    }
}
