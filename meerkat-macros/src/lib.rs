use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro that generates a `{TypeName}State` struct and a `reconstitute` method.
///
/// Apply `#[derive(Reconstitute)]` to a named-field struct to generate:
///
/// 1. A `{TypeName}State` struct with all the same fields, all `pub`
/// 2. A `pub fn reconstitute(state: {TypeName}State) -> Self` associated function
///    that maps every field from the state struct to the entity
///
/// # Example
///
/// ```ignore
/// #[derive(Reconstitute)]
/// pub struct Application {
///     id: ApplicationId,
///     name: String,
///     version: Version,
/// }
/// ```
///
/// Generates:
///
/// ```ignore
/// pub struct ApplicationState {
///     pub id: ApplicationId,
///     pub name: String,
///     pub version: Version,
/// }
///
/// impl Application {
///     pub fn reconstitute(state: ApplicationState) -> Self {
///         Self {
///             id: state.id,
///             name: state.name,
///             version: state.version,
///         }
///     }
/// }
/// ```
///
/// Only named-field structs are supported. Enums and tuple structs will
/// produce a clear compile error.
#[proc_macro_derive(Reconstitute, attributes(reconstitute_ignore))]
pub fn derive_reconstitute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let state_name = syn::Ident::new(&format!("{}State", struct_name), struct_name.span());
    let visapi = &input.vis;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Reconstitute only supports structs with named fields, not tuple structs",
                )
                    .to_compile_error()
                    .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(
                    struct_name,
                    "Reconstitute only supports structs with named fields, not unit structs",
                )
                    .to_compile_error()
                    .into();
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Reconstitute only supports structs, not enums",
            )
                .to_compile_error()
                .into();
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(
                struct_name,
                "Reconstitute only supports structs, not unions",
            )
                .to_compile_error()
                .into();
        }
    };

    let state_fields = fields.iter().filter(|f| {
        !f.attrs.iter().any(|attr| attr.path().is_ident("reconstitute_ignore"))
    }).map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { pub #name: #ty }
    });

    let field_mappings = fields.iter().map(|f| {
        let name = &f.ident;
        if f.attrs.iter().any(|attr| attr.path().is_ident("reconstitute_ignore")) {
            quote! { #name: Default::default() }
        } else {
            quote! { #name: state.#name }
        }
    });

    let expanded = quote! {
        #visapi struct #state_name {
            #(#state_fields,)*
        }

        impl #struct_name {
            pub fn reconstitute(state: #state_name) -> Self {
                Self {
                    #(#field_mappings,)*
                }
            }
        }
    };

    expanded.into()
}

#[proc_macro]
pub fn uuid_id(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct #name(::uuid::Uuid);

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #name {
            pub fn new() -> Self {
                Self(::uuid::Uuid::new_v4())
            }

            pub fn from_uuid(uuid: ::uuid::Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> &::uuid::Uuid {
                &self.0
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn slug_id(input: TokenStream) -> TokenStream {
    let name = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #name(String);

        impl #name {
            pub fn new(value: String) -> Result<Self, String> {
                if value.is_empty() {
                    return Err("Slug cannot be empty".to_string());
                }

                if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                    return Err("Slug can only contain lowercase alphanumeric characters and hyphens".to_string());
                }

                if value.starts_with('-') || value.ends_with('-') {
                    return Err("Slug cannot start or end with a hyphen".to_string());
                }

                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<String> for #name {
            type Error = String;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for #name {
            type Error = String;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value.to_string())
            }
        }

        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s.to_string())
            }
        }
    };

    TokenStream::from(expanded)
}