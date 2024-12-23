use proc_macro::TokenStream;
use quote::quote;
use quote::format_ident;


pub fn impl_named_fields(fields: syn::FieldsNamed, name:syn::Ident, spoiler:crate::SpoilerStatus) ->TokenStream{
    let buffer_name = format_ident!("{}Buffer",&name);
    let buffer_inner_name = format_ident!("{}BufferInner",&name);
    let message_name = format_ident!("{}Message",name);
    let message_inner_name = format_ident!("{}MessageInner",name);
    // let buffer_inner_name = format_ident("{}BufferInner");


    let mut msg_content = proc_macro2::TokenStream::new();
    let mut msg_match_content = proc_macro2::TokenStream::new();
    let mut buffer_content = proc_macro2::TokenStream::new();
    let mut view_content = proc_macro2::TokenStream::new();
    let mut get_content = proc_macro2::TokenStream::new();
    let mut get_content_final = proc_macro2::TokenStream::new();
    let mut set_content = proc_macro2::TokenStream::new();
    let mut from_value_content = proc_macro2::TokenStream::new();

    for field in fields.named.iter(){
        let field_name = field.ident.clone().unwrap();
        let field_type = field.ty.clone();
        let mut field_display_name: proc_macro2::TokenStream = quote!{None}.into();
        let buffer_type = quote! {<<#field_type as IcedForm>::Buffer as IcedFormBuffer>};

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
        msg_content.extend(quote! {#message_variant_name(#buffer_type::Message),});

        msg_match_content.extend(quote! {#message_inner_name::#message_variant_name(v)=> IcedFormBuffer::update(&mut self.state.#field_name,v),});

        // let buffer_part:proc_macro2::TokenStream =.into();
        buffer_content.extend( quote! {pub #field_name:<#field_type as IcedForm>::Buffer,});

        view_content.extend(quote! {
            data_column = data_column.push(IcedFormBuffer::view(&self.state.#field_name,#field_display_name)
                .map(|outer| outer.map(#message_inner_name::#message_variant_name).map(#message_name::Update)));

        });

        get_content.extend(quote! {let #field_name = if let Some(v) = IcedFormBuffer::get(&self.state.#field_name) {v} else {return None;};});
        get_content_final.extend(quote! {#field_name,});

        set_content.extend(quote! {IcedFormBuffer::set(&mut self.state.#field_name,value.#field_name);});

        from_value_content.extend(quote! {#field_name: #buffer_type::from_value(value.#field_name),});
    }

    let view_impl = spoiler.implement_view(&message_name, &view_content);
    let add_fields = spoiler.additional_fields();
    let add_msg = spoiler.additional_message();
    let add_msg_handle = spoiler.additional_message_handler(&message_name);
    let add_init = spoiler.additional_init();

    quote! {

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types)]
        pub enum #message_name{
            Update(#message_inner_name),
            #add_msg
        }

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types)]
        pub enum #message_inner_name{
            #msg_content
        }


        #[derive(Clone,Debug)]
        pub struct #buffer_name{
            pub state: #buffer_inner_name,
            #add_fields
        }

        #[derive(Clone,Debug)]
        pub struct #buffer_inner_name{
            #buffer_content
        }

        impl IcedFormBuffer for #buffer_name{
            type FormType = #name;
            type Message = #message_name;
            // fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,iced_forms::ActionOrUpdate<#message_name>,iced::Theme,iced::Renderer> {
            //     let mut data_column = iced::widget::Column::new();
            //     #view_content
            //
            //     let container = iced::widget::container(data_column.spacing(5))
            //     .padding(10)
            //     .center_x(iced::Length::Fill)
            //     .style(iced::widget::container::bordered_box);
            //     //.style(iced::container::rounded_box);
            //     if let Some(v) = title{
            //         iced::widget::container(iced::widget::column![
            //             iced::widget::text(v),
            //             container
            //         ])
            //         .center_x(iced::Length::Fill)
            //         .style(iced::widget::container::bordered_box)
            //         .into()
            //     }
            //     else{
            //         container.into()
            //     }
            // }
            #view_impl


            fn update(&mut self, outer_message:#message_name) {
                match outer_message{
                    #message_name::Update(message)=> match message{
                        #msg_match_content
                    },
                    #add_msg_handle
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

            fn from_value(value:#name)->Self{
                Self{
                    state: #buffer_inner_name{#from_value_content},
                    #add_init
                }
                //Self {#from_value_content}
            }

            // LSP. Y U NO see from_value?
        }

        impl Default for #buffer_name{
            fn default()->Self{
                #buffer_name::from_value(<<#buffer_name as IcedFormBuffer>::FormType as Default>::default())
            }
        }

        impl IcedForm for #name{
            type Buffer = #buffer_name;
        }

    }.into()

}
