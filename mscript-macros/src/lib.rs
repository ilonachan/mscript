use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse::ParseStream, Token, bracketed};

// #[proc_macro_derive(HelloMacro)]
// pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
//     // Construct a representation of Rust code as a syntax tree
//     // that we can manipulate
//     let ast = syn::parse(input).unwrap();

//     // Build the trait implementation
//     impl_hello_macro(&ast)
// }
mod kw {
    syn::custom_keyword!(obj);
}

struct SupertypeList {
    types: Vec<String>,
}

impl syn::parse::Parse for SupertypeList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut types = Vec::new();
        let content;
        bracketed!(content in input);
        while !content.is_empty() {
            let literal: syn::LitStr = content.parse()?;
            types.push(literal.value());
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        Ok(SupertypeList {types})
    }
}

struct ImplBlock {}

impl syn::parse::Parse for ImplBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

struct TypedefBlock {
    name: String,
    objtrait: syn::Ident,
    supertypes: SupertypeList,
    impl_type: ImplBlock,
    impl_obj: ImplBlock,
}

impl syn::parse::Parse for TypedefBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut impl_type: Option<ImplBlock> = None;
        let mut impl_obj: Option<ImplBlock> = None;
        let mut name = "".to_owned();
        let mut supertypes: Option<SupertypeList> = None;
        let mut objtrait: Option<syn::Ident> = None;

        while !input.is_empty() {
            if input.peek(Token![impl]) {
                if input.peek2(Token![type]) {
                    impl_type = Some(input.parse()?);
                } else if input.peek2(kw::obj) {
                    impl_obj = Some(input.parse()?);
                }
            } else {
                let field: syn::Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                match &field.to_string()[..] {
                    "name" => {
                        let name_token: syn::LitStr = input.parse()?;
                        name = name_token.value();
                    },
                    "objtrait" => {
                        objtrait = Some(input.parse()?);
                    },
                    "supertypes" => {
                        supertypes = Some(input.parse()?);
                    }
                    _ => ()
                }
                input.parse::<Token![;]>()?;
            }
        }

        Ok(TypedefBlock {
            impl_type : impl_type .unwrap(),
            impl_obj  : impl_obj  .unwrap(),
            supertypes: supertypes.unwrap(),
            objtrait  : objtrait  .unwrap(),
            name
        })
    }
}

#[proc_macro]
pub fn msh_type(tokens: TokenStream) -> TokenStream {
    let block: TypedefBlock = syn::parse_macro_input!(tokens);
    TokenStream::new()
}
