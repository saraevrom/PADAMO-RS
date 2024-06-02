use proc_macro::{TokenStream};
use quote::{quote, format_ident};
use syn::DeriveInput;
use quote::ToTokens;


#[proc_macro_derive(IcedForm, attributes(field_name))]
pub fn derive_iced_form(item: TokenStream) -> TokenStream{
    let parsed = syn::parse_macro_input!(item as DeriveInput);
    let name = parsed.ident;
    let message_name = format_ident!("{}Message",name);
    let interface_name = format_ident!("{}Interface",name);
    let mut interface_content = proc_macro2::TokenStream::new();
    let mut message_parse_content = proc_macro2::TokenStream::new();
    let mut message_content = proc_macro2::TokenStream::new();

    let mut sync_content = proc_macro2::TokenStream::new();
    let mut commit_content = proc_macro2::TokenStream::new();
    let mut view_content = proc_macro2::TokenStream::new();


    if let syn::Data::Struct(struct_) = parsed.data{
        if let syn::Fields::Named(fields) = struct_.fields{
            for field in fields.named.iter(){
                //println!("{:?}",item);
                //let mut display_name = None;
                // for attr in item.attrs.iter(){
                //     attr.se
                // }
                for attr in &field.attrs{
                    if let Some(field_name) = &field.ident{

                        if let syn::Meta::List(v) = &attr.meta{
                            if v.path.is_ident("field_name"){
                                let args = v.tokens.clone();
                                //println!("{:?}",args);
                                //println!("{:?}",field.ty);
                                let is_flag = match &field.ty {
                                    syn::Type::Path(type_path) if type_path.clone().into_token_stream().to_string() == "bool" => {
                                        true
                                    }
                                    _ => false
                                };
                                println!("{}",is_flag);
                                let field_ty = if is_flag{
                                    quote!{bool}
                                }
                                else{
                                    let inner = field.ty.clone();
                                    quote!{padamo_iced_forms::double_entry_state::EntryState<#inner>}
                                };

                                let msg_ty = if is_flag{
                                    quote!{bool}
                                }
                                else{
                                    quote!{String}
                                };



                                let msg_variant = format_ident!("Set{}",field_name);


                                interface_content.extend(quote!{
                                    #field_name : #field_ty,
                                });


                                if is_flag{

                                    message_parse_content.extend(quote!{
                                        #message_name::#msg_variant(v)=>{
                                            self.#field_name = v.clone();
                                            self.submit(target);
                                        },
                                    });
                                }
                                else{

                                    message_parse_content.extend(quote!{
                                        #message_name::#msg_variant(v)=>{
                                            self.#field_name.set_string(v.clone());
                                        },
                                    });
                                }

                                message_content.extend(quote!{
                                    #msg_variant(#msg_ty),
                                });

                                if is_flag{
                                    sync_content.extend(quote!{
                                        self.#field_name = parent.#field_name;
                                    });
                                    commit_content.extend(quote!{
                                        target.#field_name = self.#field_name;
                                    });
                                    view_content.extend(quote!{
                                        iced::widget::checkbox(#args,self.#field_name).on_toggle(#message_name::#msg_variant),
                                    });
                                }
                                else{
                                    sync_content.extend(quote!{
                                        self.#field_name.set_value(parent.#field_name.clone());
                                    });
                                    commit_content.extend(quote!{
                                        // if let Ok(v) = self.#field_name.parse(){
                                        //     target.#field_name = v;
                                        // }
                                        target.#field_name = self.#field_name.parsed_value.clone();
                                    });
                                    view_content.extend(quote!{
                                        self.#field_name.view_row(#args,"",#message_name::#msg_variant),
                                        // iced::widget::row![
                                        //     iced::widget::text(#args),
                                        //     iced::widget::TextInput::new("", &self.#field_name).width(100).on_input(#message_name::#msg_variant).on_submit(#message_name::Submit)
                                        // ],
                                    });
                                }

                                continue
                            }
                        }
                        panic!("Form must have field_name(\"...\") attribute");
                    }
                    else{
                        return quote!{compile_error!("No name in identifier")}.into()
                    }
                }
            }
            let res:TokenStream = quote!{

                #[derive(Clone,Default)]
                pub struct #interface_name {
                    #interface_content
                }

                impl padamo_iced_forms::IcedForm for #name{
                    type Interface = #interface_name;
                }


                impl padamo_iced_forms::IcedFormInterface for #interface_name{
                    type ParentType = #name;
                    type MessageType = #message_name;

                    fn new(parent:&Self::ParentType)->Self{
                        let mut res:Self = Default::default();
                        res.sync_fields(parent);
                        res
                    }

                    fn sync_fields(&mut self, parent:&Self::ParentType){
                        #sync_content
                    }

                    fn commit_fields(&self, target:&mut Self::ParentType){
                        #commit_content
                    }

                    fn submit(&mut self, target:&mut Self::ParentType){
                        self.commit_fields(target);
                        self.sync_fields(target);
                    }

                    fn update(&mut self, msg:Self::MessageType, target:&mut Self::ParentType){
                        match msg{
                            #message_parse_content
                        };
                        self.commit_fields(target);
                    }

                    fn view(&self)->iced::Element<'_,Self::MessageType>{
                        iced::widget::column![
                            #view_content
                        ].into()
                    }
                }

                #[derive(Clone,Debug)]
                pub enum #message_name{
                    //Submit,
                    #message_content
                }
            }.into();
            println!("{}",res.to_string());
            res
        }
        else{
            quote!{compile_error!("Only structures with named fields can be turned into forms for now")}.into()
        }
    }
    else{
        quote!{compile_error!("Only structures can be turned into forms now")}.into()
    }


}
