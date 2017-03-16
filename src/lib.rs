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
use syn::Ident;

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
            let mut resulting_quote = quote!();
            let field_idents: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();

//            let field_ident = &fields[0].ident;
//            let test_quote = quote!(Value::from(self.#field_ident));
            quote! {
                impl Insertable for #name {
                    fn get_msgpack_representation(&self) -> Vec<Value> {
                        let mut result : Vec<Value> = Vec::new();
                        #(
                            result.push(Value::from(self.#field_idents.clone()));
                        )*
                        result
                    }
                }
            }
        },
        _ => panic!("#[derive(new)] can only be used with structs"),
    }
}