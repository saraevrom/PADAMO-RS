use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use quote::format_ident;
use syn::{spanned::Spanned, FieldsNamed};


pub trait StructContent {

}

pub fn impl_named_fields(fields: syn::FieldsNamed, name:syn::Ident) ->TokenStream{
    let buffer_name = format_ident!("{}Buffer",&name);
    let message_name = format_ident!("{}Message",name);
    // let buffer_inner_name = format_ident("{}BufferInner");


    let mut msg_content = proc_macro2::TokenStream::new();
    let mut msg_match_content = proc_macro2::TokenStream::new();
    let mut buffer_content = proc_macro2::TokenStream::new();
    let mut view_content = proc_macro2::TokenStream::new();
    let mut get_content = proc_macro2::TokenStream::new();
    let mut get_content_final = proc_macro2::TokenStream::new();
    let mut set_content = proc_macro2::TokenStream::new();

    for field in fields.named.iter(){
        let field_name = field.ident.clone().unwrap();
        let field_type = field.ty.clone();
        let mut field_display_name: proc_macro2::TokenStream = quote!{None}.into();

        for attr in field.attrs.iter(){

            if let syn::Meta::List(v) = &attr.meta{
                if v.path.is_ident("field_name"){
                    let args = v.tokens.clone();
                    field_display_name = quote! {Some(#args)}.into();
                }
            }
        }

        // let msg_name = format!("Set{}",field_name);
        // let message_variant_name = syn::Ident::new(&msg_name, field_type.span());
        let message_variant_name = format_ident!("Set{}",field_name);

        // let msg_part:proc_macro2::TokenStream = .into();
        msg_content.extend(quote! {#message_variant_name(<<#field_type as IcedForm>::Buffer as IcedFormBuffer>::Message),});

        msg_match_content.extend(quote! {#message_name::#message_variant_name(v)=> IcedFormBuffer::update(&mut self.#field_name,v),});

        // let buffer_part:proc_macro2::TokenStream =.into();
        buffer_content.extend( quote! {pub #field_name:<#field_type as IcedForm>::Buffer,});

        view_content.extend(quote! {data_column = data_column.push(IcedFormBuffer::view(&self.#field_name,#field_display_name).map(#message_name::#message_variant_name));});

        get_content.extend(quote! {let #field_name = if let Some(v) = IcedFormBuffer::get(&self.#field_name) {v} else {return None;};});
        get_content_final.extend(quote! {#field_name,});

        set_content.extend(quote! {IcedFormBuffer::set(&mut self.#field_name,value.#field_name);});

    }
    let res = quote! {
        #[derive(Clone,Debug)]
        pub enum #message_name{
            #msg_content
        }

        #[derive(Clone,Debug,Default)]
        pub struct #buffer_name{
            #buffer_content
        }

        impl IcedFormBuffer for #buffer_name{
            type FormType = #name;
            type Message = #message_name;
            fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,#message_name,iced::Theme,iced::Renderer> {
                let mut data_column = iced::widget::Column::new();
                #view_content

                let container = iced::widget::container(data_column)
                .padding(10)
                .center_x(iced::Length::Fill)
                .style(iced::widget::container::bordered_box);
                //.style(iced::container::rounded_box);
                if let Some(v) = title{
                    iced::widget::container(iced::widget::column![
                        iced::widget::text(v),
                        container
                    ])
                    .center_x(iced::Length::Fill)
                    .style(iced::widget::container::bordered_box)
                    .into()
                }
                else{
                    container.into()
                }
            }

            fn update(&mut self, message:#message_name) {
                match message{
                    #msg_match_content
                }
            }

            fn get(&self)->Option<#name> {
                #get_content
                Some(#name{
                    #get_content_final
                })
            }

            fn set(&mut self, value:#name){
                #set_content
            }

        }

        impl IcedForm for #name{
            type Buffer = #buffer_name;
        }

    }.into();
    // println!("{}",res);
    res
}
