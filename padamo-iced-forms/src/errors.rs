use thiserror::Error;


#[derive(Clone,Debug,Error)]
pub enum FormError{
    #[error("{0}")]
    Message(String),
    #[error("{0} > {1}")]
    Outer(String,Box<FormError>)
}

impl FormError{
    pub fn map<T:Into<String>>(self,outer:T)->Self{
        Self::Outer(outer.into(), Box::new(self))
    }
}


pub type Result<T> = std::result::Result<T,FormError>;
