use proc_macro::{TokenStream};
use quote::{quote, format_ident};
use syn::DeriveInput;

#[proc_macro_attribute]
pub fn impl_content(_args:TokenStream,item:TokenStream)->TokenStream{
    let item2: proc_macro2::TokenStream = item.clone().into();
    let parsed = syn::parse_macro_input!(item as DeriveInput);
    let name = parsed.ident;
    let typename = format_ident!("{}Type",name);
    let containername = format_ident!("{}Container",name);

    if let syn::Data::Enum(enum_) = parsed.data{

        let mut type_content = proc_macro2::TokenStream::new();
        let mut impl_content = proc_macro2::TokenStream::new();
        let mut check_content = proc_macro2::TokenStream::new();
        let mut container_content = proc_macro2::TokenStream::new();
        let mut intos = proc_macro2::TokenStream::new();

        for var in enum_.variants.iter(){
            let bare_identifier = &var.ident.clone();
            let id_name = bare_identifier.to_string();
            let id_name = id_name.to_lowercase();
            let id_name = syn::Ident::new(&id_name, proc_macro2::Span::call_site());

            let function_name = format_ident!("request_{}",id_name);
            println!("{} {}",bare_identifier, function_name);

            let mut contained = None;
            if let syn::Fields::Unnamed(fs) = &var.fields{
                if let Some(i) =  fs.unnamed.first(){
                    if let syn::Type::Path(ty) = &i.ty{
                        if let Some(first_segment) = ty.path.segments.first(){
                            //println!("{:?}",first_segment.ident);
                            contained = Some(first_segment.ident.clone());
                        }
                    }
                }
            }
            if let Some(type_id) = contained{
                impl_content.extend(quote!{
                    pub fn #function_name(&self)->Result<#type_id,ExecutionError>{
                        if let Self::#bare_identifier(x) = self{
                            Ok(x.clone())
                        }
                        else{
                            Err(ExecutionError::TypeError)
                        }
                    }
                });

                container_content.extend(quote!{
                    pub fn #function_name(&self,key:&str)->Result<#type_id,ExecutionError>{
                        if let Some(x) = self.0.get(key){
                            x.#function_name()
                        }
                        else{
                            Err(ExecutionError::NotConnected(key.into()))
                        }
                    }
                });

                intos.extend(quote!{
                    impl Into<#name> for #type_id {
                        fn into(self)-> #name{
                            #name::#bare_identifier(self)
                        }
                    }
                });
            }
            else{
                return quote!{compile_error!("unnamed fields with type is required")}.into();
            }


            check_content.extend(quote!{
                (Self::#bare_identifier(_), #typename::#bare_identifier)=>true,
            });

            type_content.extend(quote!{
                #bare_identifier,
            });
        }

        quote!{
            #item2

            #intos

            impl #name {
                #impl_content

                pub fn request_type(&self, ty: &#typename)->Result<Self,ExecutionError>{
                    if self.is_compatible(ty){
                        Ok(self.clone())
                    }
                    else{
                        Err(ExecutionError::TypeError)
                    }
                }

                pub fn is_compatible(&self, ty: &#typename)->bool{
                    match (self,ty){
                        #check_content
                        _=>false,
                    }
                }
            }

            #[repr(C)]
            #[derive(abi_stable::StableAbi,Copy,Clone,Debug,Eq,PartialEq, EnumIter)]
            pub enum #typename{
                #type_content
            }




            impl #containername{
                pub fn new()->Self{
                    Self(RHashMap::new())
                }

                #container_content

                pub fn request_type(&self, ty: &#typename, key:&str)->Result<#name,ExecutionError>{
                    if let Some(x) = self.0.get(key){
                        x.request_type(ty)
                    }
                    else{
                        Err(ExecutionError::NotConnected(key.into()))
                    }
                }
            }

        }.into()
    }
    else{
        println!("trigger error");
        quote!{compile_error!("impl_content only works with enums")}.into()
    }
}
