use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Fields, FieldsUnnamed, Item, ItemEnum, ItemStruct, Lit, LitStr, Meta, MetaNameValue,
    Result, Token,
};
use turbo_tasks_macros_shared::{get_ref_ident, get_register_value_type_ident};

fn get_read_ref_ident(ident: &Ident) -> Ident {
    Ident::new(&(ident.to_string() + "ReadRef"), ident.span())
}

fn get_value_type_ident(ident: &Ident) -> Ident {
    Ident::new(
        &format!("{}_VALUE_TYPE", ident.to_string().to_uppercase()),
        ident.span(),
    )
}

fn get_value_type_id_ident(ident: &Ident) -> Ident {
    Ident::new(
        &format!("{}_VALUE_TYPE_ID", ident.to_string().to_uppercase()),
        ident.span(),
    )
}

fn get_value_type_init_ident(ident: &Ident) -> Ident {
    Ident::new(
        &format!("{}_VALUE_TYPE_INIT", ident.to_string().to_uppercase()),
        ident.span(),
    )
}

enum IntoMode {
    None,
    New,
    Shared,
}

impl Parse for IntoMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<LitStr>()?;
        Self::try_from(ident)
    }
}

impl TryFrom<LitStr> for IntoMode {
    type Error = Error;

    fn try_from(lit: LitStr) -> std::result::Result<Self, Self::Error> {
        match lit.value().as_str() {
            "none" => Ok(IntoMode::None),
            "new" => Ok(IntoMode::New),
            "shared" => Ok(IntoMode::Shared),
            _ => Err(Error::new_spanned(
                &lit,
                "expected \"none\", \"new\" or \"shared\"",
            )),
        }
    }
}

enum CellMode {
    New,
    Shared,
}

impl Parse for CellMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<LitStr>()?;
        Self::try_from(ident)
    }
}

impl TryFrom<LitStr> for CellMode {
    type Error = Error;

    fn try_from(lit: LitStr) -> std::result::Result<Self, Self::Error> {
        match lit.value().as_str() {
            "new" => Ok(CellMode::New),
            "shared" => Ok(CellMode::Shared),
            _ => Err(Error::new_spanned(&lit, "expected \"new\" or \"shared\"")),
        }
    }
}

enum SerializationMode {
    None,
    Auto,
    AutoForInput,
    Custom,
    CustomForInput,
}

impl Parse for SerializationMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<LitStr>()?;
        Self::try_from(ident)
    }
}

impl TryFrom<LitStr> for SerializationMode {
    type Error = Error;

    fn try_from(lit: LitStr) -> std::result::Result<Self, Self::Error> {
        match lit.value().as_str() {
            "none" => Ok(SerializationMode::None),
            "auto" => Ok(SerializationMode::Auto),
            "auto_for_input" => Ok(SerializationMode::AutoForInput),
            "custom" => Ok(SerializationMode::Custom),
            "custom_for_input" => Ok(SerializationMode::CustomForInput),
            _ => Err(Error::new_spanned(
                &lit,
                "expected \"none\", \"auto\", \"auto_for_input\", \"custom\" or \
                 \"custom_for_input\"",
            )),
        }
    }
}

struct ValueArguments {
    serialization_mode: SerializationMode,
    into_mode: IntoMode,
    cell_mode: CellMode,
    manual_eq: bool,
    transparent: bool,
}

impl Parse for ValueArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = ValueArguments {
            serialization_mode: SerializationMode::Auto,
            into_mode: IntoMode::None,
            cell_mode: CellMode::Shared,
            manual_eq: false,
            transparent: false,
        };
        let punctuated: Punctuated<Meta, Token![,]> = input.parse_terminated(Meta::parse)?;
        for meta in punctuated {
            match (
                meta.path()
                    .get_ident()
                    .map(ToString::to_string)
                    .as_deref()
                    .unwrap_or_default(),
                meta,
            ) {
                ("shared", Meta::Path(_)) => {
                    result.into_mode = IntoMode::Shared;
                    result.cell_mode = CellMode::Shared;
                }
                (
                    "into",
                    Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(str), ..
                    }),
                ) => {
                    result.into_mode = IntoMode::try_from(str)?;
                }
                (
                    "serialization",
                    Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(str), ..
                    }),
                ) => {
                    result.serialization_mode = SerializationMode::try_from(str)?;
                }
                (
                    "cell",
                    Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(str), ..
                    }),
                ) => {
                    result.cell_mode = CellMode::try_from(str)?;
                }
                (
                    "eq",
                    Meta::NameValue(MetaNameValue {
                        lit: Lit::Str(str), ..
                    }),
                ) => {
                    result.manual_eq = if str.value() == "manual" {
                        true
                    } else {
                        return Err(Error::new_spanned(&str, "expected \"manual\""));
                    };
                }
                ("transparent", Meta::Path(_)) => {
                    result.transparent = true;
                }
                (_, meta) => {
                    return Err(Error::new_spanned(
                        &meta,
                        format!(
                            "unexpected {:?}, expected \"shared\", \"into\", \"serialization\", \
                             \"cell\", \"eq\", \"transparent\"",
                            meta
                        ),
                    ))
                }
            }
        }

        Ok(result)
    }
}

pub fn value(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    let ValueArguments {
        serialization_mode,
        into_mode,
        cell_mode,
        manual_eq,
        transparent,
    } = parse_macro_input!(args as ValueArguments);

    let (vis, ident) = match &item {
        Item::Enum(ItemEnum { vis, ident, .. }) => (vis, ident),
        Item::Struct(ItemStruct { vis, ident, .. }) => (vis, ident),
        _ => {
            item.span().unwrap().error("unsupported syntax").emit();

            return quote! {
                #item
            }
            .into();
        }
    };

    let ref_ident = get_ref_ident(ident);
    let value_type_init_ident = get_value_type_init_ident(ident);
    let value_type_ident = get_value_type_ident(ident);
    let value_type_id_ident = get_value_type_id_ident(ident);
    let register_value_type_ident = get_register_value_type_ident(ident);

    let mut inner_type = None;
    if transparent {
        if let Item::Struct(ItemStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) = &item
        {
            if unnamed.len() == 1 {
                let field = unnamed.iter().next().unwrap();
                inner_type = Some(&field.ty);
            }
        }
    }

    let cell_update_content = match cell_mode {
        CellMode::New => quote! {
            cell.update_shared(content);
        },
        CellMode::Shared => quote! {
            // TODO we could offer a From<&#ident> when #ident implemented Clone
            cell.compare_and_update_shared(content);
        },
    };

    let (cell_prefix, cell_convert_content, cell_access_content) = if inner_type.is_some() {
        (
            quote! { pub },
            quote! {
                let content = #ident(content);
            },
            quote! {
                content.0
            },
        )
    } else {
        (
            if let IntoMode::New | IntoMode::Shared = into_mode {
                quote! { pub }
            } else {
                quote! {}
            },
            quote! {},
            quote! { content },
        )
    };

    let cell_struct = quote! {
        /// Places a value in a cell of the current task.
        ///
        /// Cell is selected based on the value type and call order of `cell`.
        #cell_prefix fn cell(self) -> turbo_tasks::vc::Vc<Self> {
            let content = self;
            turbo_tasks::vc::Vc::cell(#cell_access_content)
        }
    };

    let derive = match serialization_mode {
        SerializationMode::None | SerializationMode::Custom | SerializationMode::CustomForInput => {
            quote! {
                #[derive(turbo_tasks::trace::TraceRawVcs)]
            }
        }
        SerializationMode::Auto | SerializationMode::AutoForInput => quote! {
            #[derive(turbo_tasks::trace::TraceRawVcs, serde::Serialize, serde::Deserialize)]
        },
    };
    let debug_derive = if inner_type.is_some() {
        // Transparent structs have their own manual `ValueDebug` implementation.
        quote! {
            #[repr(transparent)]
        }
    } else {
        quote! {
            #[derive(turbo_tasks::debug::ValueDebugFormat, turbo_tasks::debug::internal::ValueDebug)]
        }
    };
    let eq_derive = if manual_eq {
        quote!()
    } else {
        quote!(
            #[derive(PartialEq, Eq)]
        )
    };

    let new_value_type = match serialization_mode {
        SerializationMode::None => quote! {
            turbo_tasks::ValueType::new::<#ident>()
        },
        SerializationMode::Auto | SerializationMode::Custom => {
            quote! {
                turbo_tasks::ValueType::new_with_any_serialization::<#ident>()
            }
        }
        SerializationMode::AutoForInput | SerializationMode::CustomForInput => {
            quote! {
                turbo_tasks::ValueType::new_with_magic_serialization::<#ident>()
            }
        }
    };

    let for_input_marker = match serialization_mode {
        SerializationMode::None | SerializationMode::Auto | SerializationMode::Custom => quote! {},
        SerializationMode::AutoForInput | SerializationMode::CustomForInput => quote! {
            impl turbo_tasks::TypedForInput for #ident {}
        },
    };

    let value_debug_impl = if inner_type.is_some() {
        // For transparent values, we defer directly to the inner type's `ValueDebug`
        // implementation.
        quote! {
            #[turbo_tasks::value_impl]
            impl turbo_tasks::debug::ValueDebug for #ident {
                #[turbo_tasks::function]
                async fn dbg(&self) -> anyhow::Result<turbo_tasks::debug::ValueDebugStringVc> {
                    use turbo_tasks::debug::ValueDebugFormat;
                    (&self.0).value_debug_format().try_to_value_debug_string().await
                }
            }
        }
    } else {
        quote! {}
    };

    let inner = if let Some(inner_type) = inner_type {
        quote! {
            #inner_type
        }
    } else {
        quote! {
            #ident
        }
    };

    let (read, strongly_consistent_read) = if let Some(inner_type) = inner_type {
        (
            quote! {
                /// Safety: Types are binary identical via #[repr(transparent)]
                unsafe { node.into_transparent_read::<Self, Self::Inner>() }
            },
            quote! {
                /// Safety: Types are binary identical via #[repr(transparent)]
                unsafe { node.into_transparent_strongly_consistent_read::<#ident, #inner_type>() }
            },
        )
    } else {
        (
            quote! { node.into_read::<Self>() },
            quote! { node.into_strongly_consistent_read::<#ident>() },
        )
    };

    let expanded = quote! {
        #derive
        #eq_derive
        #debug_derive
        #item

        impl #ident {
            #cell_struct
        }

        #[doc(hidden)]
        static #value_type_init_ident: turbo_tasks::macro_helpers::OnceCell<
            turbo_tasks::ValueType,
        > = turbo_tasks::macro_helpers::OnceCell::new();
        #[doc(hidden)]
        pub(crate) static #value_type_ident: turbo_tasks::macro_helpers::Lazy<&turbo_tasks::ValueType> =
            turbo_tasks::macro_helpers::Lazy::new(|| {
                #value_type_init_ident.get_or_init(|| {
                    panic!(
                        concat!(
                            stringify!(#value_type_ident),
                            " has not been initialized (this should happen via the generated register function)"
                        )
                    )
                })
            });
        #[doc(hidden)]
        static #value_type_id_ident: turbo_tasks::macro_helpers::Lazy<turbo_tasks::ValueTypeId> =
            turbo_tasks::macro_helpers::Lazy::new(|| {
                turbo_tasks::registry::get_value_type_id(*#value_type_ident)
            });
        #[doc(hidden)]
        #[allow(non_snake_case)]
        pub(crate) fn #register_value_type_ident(
            global_name: &'static str,
            f: impl FnOnce(&mut turbo_tasks::ValueType),
        ) {
            #value_type_init_ident.get_or_init(|| {
                let mut value = #new_value_type;
                f(&mut value);
                value
            }).register(global_name);
        }

        impl turbo_tasks::Typed for #ident {
            fn get_value_type_id() -> turbo_tasks::ValueTypeId {
                *#value_type_id_ident
            }
        }
        #for_input_marker

        #value_debug_impl

        #vis type #ref_ident = turbo_tasks::vc::Vc<#ident>;

        // TODO(alexkirsz) Only when shared.
        impl turbo_tasks::vc::ValueCell for #ident {}

        impl turbo_tasks::vc::Value for #ident {
            type TraitTypeIdsIterator = Box<dyn Iterator<Item = turbo_tasks::TraitTypeId>>;

            type Inner = #inner;

            fn convert(content: Self::Inner) -> Self {
                #cell_convert_content
                content
            }

            fn update(content: Self) -> turbo_tasks::RawVc {
                let cell = turbo_tasks::macro_helpers::find_cell_by_type(*#value_type_id_ident);
                #cell_update_content
                cell.into()
            }

            fn read(node: &turbo_tasks::RawVc) -> turbo_tasks::ReadRawVcFuture<Self, Self::Inner> {
                #read
            }

            fn strongly_consistent_read(node: &turbo_tasks::RawVc) -> turbo_tasks::ReadRawVcFuture<Self, Self::Inner> {
                #strongly_consistent_read
            }

            fn get_value_type_id() -> turbo_tasks::ValueTypeId {
                *#value_type_id_ident
            }

            fn get_trait_type_ids() -> Self::TraitTypeIdsIterator {
                Box::new(#value_type_ident.traits_iter())
            }
        }
    };

    expanded.into()
}
