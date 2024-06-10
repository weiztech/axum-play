use proc_macro::TokenStream;
use quote::quote;
use syn::Fields;

#[proc_macro_derive(ToSqlString)]
pub fn sql_string_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_tosqlstr_macro(&ast)
}

fn impl_tosqlstr_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = match &ast.data {
        syn::Data::Struct(ref data) => {
            let fields = match &data.fields {
                Fields::Named(field) => &field.named,
                _ => panic!("Only named fields are supported"),
            };

            let field_operations = fields.iter().map(|field| {
                let field_name = &field.ident;

                quote! {
                    if (!self.#field_name.is_none()){
                        query_str += format!(
                        "{} {} {} ${} ",
                        if query_str.len() == 0 {"WHERE"} else {separator},
                        stringify!(#field_name),
                        operator,
                        counter,
                        ).as_str();

                        counter += 1;
                        query_param.push(
                            format!("%{}%", self.#field_name.as_ref().unwrap())
                        );
                    }
                }
            });

            quote! {
                impl ToSqlString for #name {
                    fn as_sql_string(
                        &self,
                        operator: &str,
                        separator: &str,
                        order_by: &str,
                        pagination: &PaginationOptions,
                    ) -> (String, Vec<String>) {
                        let mut counter = 1;
                        let mut query_str = String::new();
                        let mut query_param: Vec<String> = Vec::new();
                        #(#field_operations)*

                        if let Some(previous) = &pagination.previous{
                            query_str += format!(
                                "{} {} < ${} ",
                                if query_str.len() == 0 {"WHERE"} else {separator},
                                "id",
                                counter,
                            ).as_str();

                            query_param.push(previous.to_string());
                            counter += 1;
                        }

                        query_str += format!(
                            "ORDER BY {} ",
                            order_by,
                        ).as_str();

                        if let Some(limit) = pagination.limit{
                            query_str += format!(
                                "limit {}",
                                limit,
                            ).as_str();
                        }
                        (query_str, query_param)
                    }
                }
            }
        }
        _ => quote! {
            compile_error!("Validate can only be used on structs");
        },
    };
    gen.into()
}
