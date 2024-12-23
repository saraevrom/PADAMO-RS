use proc_macro::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput};
mod named_fields;
mod unnamed_fields;
mod enums;


fn get_name_code(attrs:&[syn::Attribute])->proc_macro2::TokenStream{
    let mut field_display_name: proc_macro2::TokenStream = quote!{None}.into();

    for attr in attrs.iter(){

        if let syn::Meta::List(v) = &attr.meta{
            if v.path.is_ident("field_name"){
                let args = v.tokens.clone();
                field_display_name = quote! {Some(#args)}.into();
            }
        }
    }
    field_display_name
}

enum SpoilerStatus{
    None,
    Hidden,
    Shown
}

impl SpoilerStatus{
    pub fn additional_fields(&self)->proc_macro2::TokenStream{
        if let Self::None = self{
            proc_macro2::TokenStream::new()
        }
        else{
            quote! {pub visible:bool,}
        }
    }

    pub fn additional_message(&self)->proc_macro2::TokenStream{
        if let Self::None = self{
            proc_macro2::TokenStream::new()
        }
        else{
            quote! {SetVisible(bool),}
        }
    }

    pub fn additional_init(&self)->proc_macro2::TokenStream{
        match self {
            Self::None=>proc_macro2::TokenStream::new(),
            Self::Hidden=>quote! {visible:false,},
            Self::Shown=>quote! {visible:true,}
        }
    }

    pub fn additional_message_handler(&self, message_name:&syn::Ident)->proc_macro2::TokenStream{
        if let Self::None = self{
            proc_macro2::TokenStream::new()
        }
        else{
            quote! {#message_name::SetVisible(v)=>{self.visible = v},}
        }
    }

    pub fn implement_view(&self, message_name:&syn::Ident, view_content:&proc_macro2::TokenStream)->proc_macro2::TokenStream{
        if let Self::None = self{
            quote! {
                fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,padamo_iced_forms::ActionOrUpdate<#message_name>,iced::Theme,iced::Renderer> {
                    let mut data_column = iced::widget::Column::new();
                    #view_content

                    let container = iced::widget::container(data_column.spacing(5))
                    .padding(10)
                    .center_x(iced::Length::Fill)
                    .style(iced::widget::container::bordered_box);
                    //.style(iced::container::rounded_box);
                    if let Some(v) = title{
                        iced::widget::container(iced::widget::column![
                            iced::widget::text(v),
                            container
                        ])
                        .width(iced::Length::Fill)
                        // .center_x(iced::Length::Fill)
                        .style(iced::widget::container::bordered_box)
                        .into()
                    }
                    else{
                        container.into()
                    }
                }
            }
        }
        else{
            quote! {
                fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,padamo_iced_forms::ActionOrUpdate<#message_name>,iced::Theme,iced::Renderer> {
                    let mut data_column = iced::widget::Column::new();
                    #view_content

                    let container = iced::widget::container(data_column.spacing(5))
                    .padding(10)
                    .center_x(iced::Length::Fill)
                    .style(iced::widget::container::bordered_box);
                    //.style(iced::container::rounded_box);
                    if let Some(v) = title{
                        let mut inner_collumn = iced::widget::column![
                            iced::widget::text(v),
                            iced::widget::checkbox("Show",self.visible).on_toggle(|x| padamo_iced_forms::ActionOrUpdate::Update(#message_name::SetVisible(x))),
                            //container
                        ];
                        if self.visible{
                            inner_collumn = inner_collumn.push(container);
                        }

                        iced::widget::container(inner_collumn)
                            .width(iced::Length::Fill)
                            // .center_x(iced::Length::Fill)
                            .style(iced::widget::container::bordered_box)
                            .into()
                    }
                    else{
                        container.into()
                    }
                }
            }
        }
    }
}

fn get_spoiler(attrs:&[syn::Attribute])->SpoilerStatus{

    for attr in attrs.iter(){
        //println!("{:?}", attr);
        if let syn::Meta::Path(p) = &attr.meta{
            if p.is_ident("spoiler_hidden"){
                return SpoilerStatus::Hidden;
            }
            if p.is_ident("spoiler_shown"){
                return SpoilerStatus::Shown;
            }
        }
    }
    SpoilerStatus::None
}


fn impl_unit_struct(name:syn::Ident) -> TokenStream {
    let buffer_name = quote::format_ident!("{}Buffer",&name);
    let buffer_inner_name = quote::format_ident!("{}BufferInner",&name);
    let message_name = quote::format_ident!("{}Message",&name);

    quote!{
        impl IcedForm for #name{
            type Buffer = #buffer_name;
        }

        #[derive(Clone,Debug, Default)]
        pub struct #buffer_name{
            pub state: #buffer_inner_name,
        }

        #[derive(Clone,Debug, Default)]
        pub struct #buffer_inner_name;

        #[derive(Clone,Debug)]
        pub struct #message_name;

        impl IcedFormBuffer for #buffer_name{
            type Message = #message_name;
            type FormType = #name;
            fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,iced_forms::ActionOrUpdate<Self::Message>,iced::Theme,iced::Renderer> {
                if let Some(v) = title{
                    iced::widget::text(v).into()
                }
                else{
                    iced::widget::column![].into()
                }
            }
            fn update(&mut self, _message:Self::Message) {

            }

            fn get(&self)->Option<#name> {
                Some(#name)
            }

            fn set(&mut self, value:#name){

            }

            fn from_value(value:#name)->Self{
                Self{state: #buffer_inner_name}
            }
        }
    }.into()
}

fn impl_icedform_struct(input: syn::DataStruct,name:syn::Ident,spoiler:SpoilerStatus) -> TokenStream {
    match input.fields{
        syn::Fields::Unit=>impl_unit_struct(name),
        syn::Fields::Named(n)=>named_fields::impl_named_fields(n,name,spoiler),
        syn::Fields::Unnamed(un)=>unnamed_fields::impl_unnamed_fields(un, name,spoiler),
    }
}



#[proc_macro_derive(IcedForm, attributes(field_name,spoiler_hidden,spoiler_shown))]
pub fn icedform_macro_derive(input: TokenStream) -> TokenStream {
    let ast= syn::parse_macro_input!(input as DeriveInput);
    let name = ast.ident;
    let spoiler = get_spoiler(&ast.attrs);
    match ast.data{
        syn::Data::Struct(s) => impl_icedform_struct(s,name, spoiler),
        syn::Data::Enum(e) => enums::impl_icedform_enum(e, name),
        syn::Data::Union(_)=>quote!{compile_error!("Unions are not supported")}.into(),
    }
}
