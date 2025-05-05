use proc_macro::TokenStream;
use quote::quote;
use quote::format_ident;


pub fn impl_icedform_enum(input: syn::DataEnum,name:syn::Ident) -> TokenStream {
    let buffer_name = format_ident!("{}Buffer",&name);
    let buffer_content_name = format_ident!("{}BufferContent",&name);
    let message_name = format_ident!("{}Message",name);
    let choose_message_name = format_ident!("{}ChooseMessage",name);
    let update_message_name = format_ident!("{}MessageUpdate",name);
    let stash_name = format_ident!("{}Stash",&name);

    let mut message_content = proc_macro2::TokenStream::new();
    let mut choose_message_content = proc_macro2::TokenStream::new();
    // let mut buffer_content = proc_macro2::TokenStream::new();
    let mut update_content = proc_macro2::TokenStream::new();
    let mut choose_update_content = proc_macro2::TokenStream::new();
    let mut get_content = proc_macro2::TokenStream::new();
    let mut set_content = proc_macro2::TokenStream::new();

    let mut buffer_variants = proc_macro2::TokenStream::new();

    //let mut proxy_to_self_conversions = proc_macro2::TokenStream::new();

    let mut additional_impls = proc_macro2::TokenStream::new();
    let mut display_names_content = proc_macro2::TokenStream::new();

    let mut overlay_content = proc_macro2::TokenStream::new();
    let mut element_view = proc_macro2::TokenStream::new();

    let mut stash_content = proc_macro2::TokenStream::new();
    let mut stash_match_content = proc_macro2::TokenStream::new();

    let mut variant_impl_display_content = proc_macro2::TokenStream::new();


    for variant in input.variants{
        let variant_display_name = crate::get_name_code(&variant.attrs);
        let variant_name = variant.ident;
        let proxy_name = format_ident!("{}{}Proxy",name,variant_name);
        let proxy_buffer_name = format_ident!("{}{}ProxyBuffer",name,variant_name);
        let proxy_buffer_inner_name = format_ident!("{}{}ProxyBufferInner",name,variant_name);
        let proxy_message_name = format_ident!("{}{}ProxyMessage",name,variant_name);
        // let message_variant_name = format_ident!("Set{}",variant_name);


        let choose_variant_name = format_ident!("Choose{}",variant_name);

        // let message_variant:proc_macro2::TokenStream;
        // let buffer_variant:proc_macro2::TokenStream;


        let mut unpacked = proc_macro2::TokenStream::new();
        let mut unpacked_setter = proc_macro2::TokenStream::new();
        let proxy_content:proc_macro2::TokenStream;
        let mut contents_match_part = proc_macro2::TokenStream::new();

        match variant.fields{
            syn::Fields::Unit=>{
                proxy_content = quote! {;};
            },
            syn::Fields::Unnamed(un)=>{
                for (i,_) in un.unnamed.iter().enumerate(){
                    let index = syn::Index::from(i);
                    let contents_index = format_ident!("contents_{}",index);
                    unpacked.extend(quote! {self.#index,});
                    unpacked_setter.extend(quote! { IcedFormBuffer::from_value(#contents_index),});
                    contents_match_part.extend(quote! {#contents_index,});
                }
                unpacked = quote! { (#unpacked) };
                unpacked_setter = quote! { (#unpacked_setter) };
                proxy_content = quote! {#un;};
                contents_match_part = quote! {(#contents_match_part)};

            },
            syn::Fields::Named(n)=>{


                for field in n.named.iter(){
                    let field_name = field.ident.clone().unwrap();
                    unpacked.extend(quote! {#field_name: self.#field_name,});
                    unpacked_setter.extend(quote! {#field_name:IcedFormBuffer::from_value(#field_name),});
                    contents_match_part.extend(quote! {#field_name,});
                }

                unpacked = quote! { {#unpacked} };
                unpacked_setter = quote! { {#unpacked_setter} };
                contents_match_part = quote! {{#contents_match_part}};
                proxy_content = quote! {#n};

            },
        }

        additional_impls.extend(quote!{
            #[derive(IcedForm,Clone,Default)]
            #[allow(non_camel_case_types,non_snake_case)]
            pub struct #proxy_name #proxy_content

            impl std::convert::Into<#name> for #proxy_name{
                fn into(self)->#name{
                    #name::#variant_name #unpacked
                }
            }
        });


        buffer_variants.extend(quote!{#variant_name(#proxy_buffer_name),});
        choose_message_content.extend(quote!{#choose_variant_name,});
        message_content.extend(quote!{#variant_name(#proxy_message_name),});

        //buffer_content.extend(buffer_variant);
        // message_content.extend(message_variant);

        choose_update_content.extend(quote! {
            #choose_message_name::#choose_variant_name=>{
                self.expanded = false;
                self.stash_current_variant();
                // let new_value = <#proxy_buffer_name as Default>::default();
                let new_value = self.stash.#variant_name.clone().unwrap_or_default();
                self.state = #buffer_content_name::#variant_name(new_value);
            }
        });

        update_content.extend(quote! {

            #update_message_name::#variant_name(sub_msg)=>{
                if let #buffer_content_name::#variant_name(buf) = &mut self.state{
                    IcedFormBuffer::update(buf,sub_msg)
                }
            },

        });
        get_content.extend(quote! {
            #buffer_content_name::#variant_name(buf)=>{
                IcedFormBuffer::get(buf).map(|x| x.into())
            },
        });

        element_view.extend(quote! {
            #buffer_content_name::#variant_name(buf)=>{
                IcedFormBuffer::view(buf,None).map(|outer| outer.map(#update_message_name::#variant_name))
            },
        });

        let new_display_name = quote! {(#variant_display_name).unwrap_or(stringify!(#variant_name))};

        display_names_content.extend(quote! {
            #buffer_content_name::#variant_name(_)=>#new_display_name,
        });

        overlay_content.extend(quote! {
            iced::widget::Button::new(iced::widget::Text::new(#new_display_name)).width(100).on_press(#choose_message_name::#choose_variant_name),
        });

        set_content.extend(quote! {
            #name::#variant_name #contents_match_part=>#buffer_content_name::#variant_name(#proxy_buffer_name{state: #proxy_buffer_inner_name #unpacked_setter} ),
        });

        stash_content.extend(quote! {pub #variant_name:Option<#proxy_buffer_name>,});
        stash_match_content.extend(quote! {
            #buffer_content_name::#variant_name(buf)=>{
                self.stash.#variant_name = Some(buf.clone());
            },
        });

        variant_impl_display_content.extend(quote! {
            #choose_message_name::#choose_variant_name=>#new_display_name,
        });
        // set_content.extend(quote! {
        //     #name::#variant_name
        // });
    }
    quote! {

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub struct #buffer_name{
            pub expanded: bool,
            pub state: #buffer_content_name,
            //pub display_state: #choose_message_name,
            pub stash: #stash_name
        }

        impl #buffer_name{
            fn stash_current_variant(&mut self){
                match &self.state{
                    #stash_match_content
                }
            }
        }

        #[derive(Clone,Debug, Default)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub struct #stash_name{
            #stash_content
        }

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub enum #buffer_content_name{
            #buffer_variants
        }



        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub enum #message_name{
            Update(#update_message_name),
            Choose(#choose_message_name),
            Expand,
            Dismiss,
            SetExpanded(bool),
        }

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub enum #update_message_name{
            #message_content
        }

        #[derive(Clone,Debug)]
        #[allow(non_camel_case_types,non_snake_case)]
        pub enum #choose_message_name{
            #choose_message_content
        }

        impl std::fmt::Display for #choose_message_name{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    #variant_impl_display_content
                })
            }
        }

        impl IcedForm for #name{
            type Buffer = #buffer_name;
        }

        impl IcedFormBuffer for #buffer_name{
            type FormType = #name;
            type Message = #message_name;

            fn update(&mut self, full_message:#message_name) {
                match full_message{
                    #message_name::Update(message) => match message{
                        #update_content
                    },
                    #message_name::Choose(message) => match message{
                        #choose_update_content
                    }
                    #message_name::Expand =>{
                        self.expanded = true;
                    },
                    #message_name::Dismiss =>{
                        self.expanded = false;
                    },
                    #message_name::SetExpanded(v) =>{
                        self.expanded = v;
                    },
                }
            }

            fn get(&self)->padamo_iced_forms::Result<#name> {
                match &self.state{
                    #get_content
                }.map_err(|e| e.map("Selected value"))
            }

            fn set(&mut self, value:#name){
                self.state = match value{
                    #set_content
                }
            }

            fn from_value(value:#name)->Self{
                Self{
                    expanded:false,
                    stash:Default::default(),
                    state: match value{
                        #set_content
                    }
                }

            }

            fn view<'a>(&'a self,title:Option<&'a str>)->iced::Element<'a,padamo_iced_forms::ActionOrUpdate<#message_name>,iced::Theme,iced::Renderer> {
                let display_name = match &self.state{
                    #display_names_content
                };

                let label = title.unwrap_or_default();

                let underlay = iced::widget::Row::new()
                    // .push(iced::widget::Text::new(label))
                    .push(iced::widget::Button::new(iced::widget::Text::new(display_name)).width(100).on_press(#message_name::Expand));

                let overlay:iced::Element<'a,#choose_message_name,iced::Theme,iced::Renderer> = iced::widget::column![
                    #overlay_content
                ].into();
                //
                // // underlay.into()
                //
                let drop_down = iced_aw::DropDown::new(underlay, overlay.map(#message_name::Choose), self.expanded)
                    .width(iced::Length::Fill)
                    .on_dismiss(#message_name::Dismiss);
                    // .alignment(iced_aw::drop_down::Alignment::Bottom);
                //let underlay = iced::widget::checkbox(display_name, self.expanded).on_toggle(#message_name::SetExpanded);
                // let underlay = if self.expanded{
                //     iced::widget::Button::new("Dismiss").on_press(padamo_iced_forms::ActionOrUpdate::Update(#message_name::Dismiss))
                // }
                // else{
                //     iced::widget::Button::new(iced::widget::Text::new(display_name)).on_press(padamo_iced_forms::ActionOrUpdate::Update(#message_name::Expand))
                // };
                // let overlay:iced::Element<'a,#choose_message_name,iced::Theme,iced::Renderer> = iced::widget::column![
                //     #overlay_content
                // ].into();

                // let mut dropdown = iced::widget::Column::new();
                // dropdown = dropdown.push(underlay);
                // if self.expanded{
                //     dropdown = dropdown.push(overlay.map(#message_name::Choose).map(padamo_iced_forms::ActionOrUpdate::Update));
                // }


                // drop_down.into()
                let drop_down:iced::Element<'a,#message_name,iced::Theme, iced::Renderer> = drop_down.into();

                let selected_element:iced::Element<'a,padamo_iced_forms::ActionOrUpdate<#update_message_name>,iced::Theme,iced::Renderer> = match &self.state{
                    #element_view
                }.into();

                let selected_element = selected_element.map(|outer| outer.map(#message_name::Update));
                let res = iced::widget::column![
                    iced::widget::Text::new(label),
                    drop_down.map(padamo_iced_forms::ActionOrUpdate::Update),
                    // iced::widget::row![,drop_down].spacing(5),
                    selected_element
                ];
                iced::widget::container(res)
                    .padding(10)
                    // .center_x(iced::Length::Fill)
                    .width(iced::Length::Fill)
                    .style(iced::widget::container::bordered_box)
                    .into()
                // drop_down.into()
            }
        }

        impl Default for #buffer_name{
            fn default()->Self{
                #buffer_name::from_value(<<#buffer_name as IcedFormBuffer>::FormType as Default>::default())
            }
        }

        #additional_impls
    }.into()
}
