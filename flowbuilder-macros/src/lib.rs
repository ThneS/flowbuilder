//! # FlowBuilder Macros
//!
//! Procedural macros for easier flow definition and step creation

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemFn, LitStr};

/// Attribute macro to convert a function into a flow step
///
/// # Example
/// ```rust,ignore
/// use flowbuilder_macros::step;
/// use flowbuilder_context::SharedContext;
///
/// #[step]
/// async fn my_step(ctx: SharedContext) -> anyhow::Result<()> {
///     println!("Executing step");
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn step(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_generics = &input_fn.sig.generics;
    let fn_asyncness = &input_fn.sig.asyncness;

    let expanded = quote! {
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            #fn_body
        }
    };

    TokenStream::from(expanded)
}

/// Attribute macro to convert a function into a named flow step
///
/// # Example
/// ```rust,ignore
/// use flowbuilder_macros::named_step;
/// use flowbuilder_context::SharedContext;
///
/// #[named_step("my_step")]
/// async fn my_step(ctx: SharedContext) -> anyhow::Result<()> {
///     println!("Executing my_step");
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn named_step(attr: TokenStream, item: TokenStream) -> TokenStream {
    let step_name = parse_macro_input!(attr as LitStr);
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_generics = &input_fn.sig.generics;
    let fn_asyncness = &input_fn.sig.asyncness;

    let expanded = quote! {
        #fn_vis #fn_asyncness fn #fn_name #fn_generics(#fn_inputs) #fn_output {
            // Auto-generate step logging
            let step_name = #step_name;
            let ctx_clone = ctx.clone();

            // Start step logging
            {
                let mut guard = ctx_clone.lock().await;
                guard.start_step(step_name.to_string());
            }

            let result = async move #fn_body.await;

            // End step logging
            {
                let mut guard = ctx_clone.lock().await;
                match &result {
                    Ok(()) => guard.end_step_success(step_name),
                    Err(e) => guard.end_step_failed(step_name, &e.to_string()),
                }
            }

            result
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for creating flow context variables
///
/// # Example
/// ```rust,ignore
/// use flowbuilder_macros::FlowContext;
///
/// #[derive(FlowContext)]
/// struct MyContext {
///     user_id: String,
///     session_token: String,
/// }
/// ```
#[proc_macro_derive(FlowContext)]
pub fn derive_flow_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let data = &input.data;

    let fields = match data {
        syn::Data::Struct(data_struct) => &data_struct.fields,
        _ => panic!("FlowContext can only be derived for structs"),
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let getters = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            let getter_name = syn::Ident::new(&format!("get_{}", name), name.span());
            quote! {
                pub fn #getter_name(&self, ctx: &FlowContext) -> Option<#ty> {
                    ctx.get_variable(stringify!(#name))
                        .and_then(|v| v.parse().ok())
                }
            }
        });

    let setters = field_names
        .iter()
        .zip(field_types.iter())
        .map(|(name, ty)| {
            let setter_name = syn::Ident::new(&format!("set_{}", name), name.span());
            quote! {
                pub fn #setter_name(&self, ctx: &mut FlowContext, value: #ty) {
                    ctx.set_variable(stringify!(#name).to_string(), value.to_string());
                }
            }
        });

    let expanded = quote! {
        impl #name {
            #(#getters)*
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}

/// Macro for creating flow DSL
///
/// # Example
/// ```rust,ignore
/// use flowbuilder_macros::flow;
/// use flowbuilder_core::FlowBuilder;
///
/// let my_flow = flow! {
///     step "init" => |ctx| async move {
///         println!("Initializing");
///         Ok(())
///     },
///     step "process" => |ctx| async move {
///         println!("Processing");
///         Ok(())
///     },
///     step_if "cleanup" when |ctx| ctx.get_variable("needs_cleanup").is_some() => |ctx| async move {
///         println!("Cleaning up");
///         Ok(())
///     }
/// };
/// ```
#[proc_macro]
pub fn flow(_input: TokenStream) -> TokenStream {
    // 这里可以实现一个简单的DSL解析
    // 暂时返回一个基本的FlowBuilder
    let expanded = quote! {
        FlowBuilder::new()
    };

    TokenStream::from(expanded)
}
