#![crate_type = "proc-macro"]

extern crate proc_macro;
extern crate syn;
extern crate rmpv;
#[macro_use]
extern crate quote;
extern crate tarantool;

use proc_macro::TokenStream;
use rmpv::Value;
use tarantool::Insertable;

#[proc_macro_derive(Insertable)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();

    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");

    let result = new_for_struct(ast);

    result.to_string().parse().expect("couldn't parse string to tokens")
}

fn new_for_struct(ast: syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let doc_comment = format!("Constructs a new `{}`.", name);

    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            let vector_body = fields.iter().map(|f| {
                let f_name = &f.ident;
                quote!(#(Value::from(self.f_name),)*)
            });
            println!("vector body: {:?}", vector_body);

            quote! {
                impl Insertable for #name #ty_generics #where_clause {
                    fn get_msgpack_representation(&self) -> Vec<Value> {
                        vec![#(vector_body),*]
                    }
                }
            }
        },
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}