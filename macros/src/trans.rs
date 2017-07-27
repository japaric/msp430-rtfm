use quote::{Ident, Tokens};
use syn::{Lit, StrStyle};

use analyze::Ownerships;
use check::App;

fn krate() -> Ident {
    Ident::from("rtfm")
}

pub fn app(app: &App, ownerships: &Ownerships) -> Tokens {
    let mut root = vec![];
    let mut main = vec![];

    ::trans::check(app, &mut main);
    ::trans::init(app, &mut main, &mut root);
    ::trans::idle(app, ownerships, &mut main, &mut root);
    ::trans::resources(app, ownerships, &mut root);
    ::trans::tasks(app, &mut root);

    root.push(quote! {
        #[allow(unsafe_code)]
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

// Sadly we can't do these tests at expansion time. Instead we'll generate some
// code that won't compile if the test fails.
fn check(app: &App, main: &mut Vec<Tokens>) {
    let device = &app.device;

    // Checks that the interrupts are valid
    for task in app.tasks.keys() {
        main.push(quote! {
            let _ = #device::Interrupt::#task;
        });
    }

    // Checks that all the resource data implements the `Send` trait
    if !app.resources.is_empty() {
        main.push(quote! {
            fn is_send<T>() where T: Send {}
        });
    }

    for resource in app.resources.values() {
        let ty = &resource.ty;

        main.push(quote!(is_send::<#ty>();));
    }
}

fn idle(
    app: &App,
    ownerships: &Ownerships,
    main: &mut Vec<Tokens>,
    root: &mut Vec<Tokens>,
) {
    let krate = krate();

    let mut mod_items = vec![];
    let mut tys = vec![];
    let mut exprs = vec![];

    if !app.idle.resources.is_empty() {
        let device = &app.device;

        let mut needs_reexport = false;
        for name in &app.idle.resources {
            if ownerships[name].is_owned() {
                if app.resources.get(name).is_some() {
                    // it's not a peripheral
                    needs_reexport = true;
                    break;
                }
            }
        }

        let super_ = if needs_reexport {
            None
        } else {
            Some(Ident::new("super"))
        };
        let mut rexprs = vec![];
        let mut rfields = vec![];
        for name in &app.idle.resources {
            let _name = Ident::new(format!("_{}", name.as_ref()));

            if ownerships[name].is_owned() {
                if let Some(resource) = app.resources.get(name) {
                    let ty = &resource.ty;

                    rfields.push(quote! {
                        pub #name: &'static mut #ty,
                    });

                    rexprs.push(quote! {
                        #name: ::#krate::Static::ref_mut(
                            &mut #super_::#_name,
                        ),
                    });
                } else {
                    rfields.push(quote! {
                        pub #name: &'static mut ::#device::#name,
                    });

                    rexprs.push(quote! {
                        #name: #krate::Static::ref_mut(
                            &mut *::#device::#name.get(),
                        ),
                    });
                }
            } else {
                rfields.push(quote! {
                    pub #name: #super_::_resource::#name,
                });

                rexprs.push(quote! {
                    #name: #super_::_resource::#name::new(),
                });
            }
        }

        if needs_reexport {
            root.push(quote! {
                #[allow(non_camel_case_types)]
                #[allow(non_snake_case)]
                pub struct _idleResources {
                    #(#rfields)*
                }
            });

            mod_items.push(quote! {
                pub use ::_idleResources as Resources;
            });
        } else {
            mod_items.push(quote! {
                #[allow(non_snake_case)]
                pub struct Resources {
                    #(#rfields)*
                }
            });
        }

        mod_items.push(quote! {
            #[allow(unsafe_code)]
            impl Resources {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#rexprs)*
                    }
                }
            }
        });

        tys.push(quote!(&mut #krate::Threshold));
        tys.push(quote!(idle::Resources));

        exprs.push(quote!(unsafe { &mut #krate::Threshold::new(0) }));
        exprs.push(quote!(unsafe { idle::Resources::new() }));
    }

    if !mod_items.is_empty() {
        root.push(quote! {
            #[allow(unsafe_code)]
            mod idle {
                #(#mod_items)*
            }
        });
    }

    let idle = &app.idle.path;
    main.push(quote! {
        // type check
        let idle: fn(#(#tys),*) -> ! = #idle;

        idle(#(#exprs),*);
    });
}

fn init(app: &App, main: &mut Vec<Tokens>, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let init = &app.init.path;
    let krate = krate();

    let mut tys = vec![quote!(#device::Peripherals)];
    let mut exprs = vec![quote!(unsafe { #device::Peripherals::all() })];
    let mut mod_items = vec![];

    if !app.resources.is_empty() {
        let mut fields = vec![];
        let mut rexprs = vec![];

        for (name, resource) in &app.resources {
            let _name = Ident::new(format!("_{}", name.as_ref()));
            let ty = &resource.ty;

            fields.push(quote! {
                pub #name: &'a mut #krate::Static<#ty>,
            });

            rexprs.push(quote! {
                #name: ::#krate::Static::ref_mut(&mut super::#_name),
            });
        }

        root.push(quote! {
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            pub struct _initResources<'a> {
                #(#fields)*
            }
        });

        mod_items.push(quote! {
            pub use ::_initResources as Resources;

            impl<'a> Resources<'a> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#rexprs)*
                    }
                }
            }
        });

        tys.push(quote!(init::Resources));
        exprs.push(quote!(unsafe { init::Resources::new() }));
    }

    root.push(quote! {
        #[allow(unsafe_code)]
        mod init {
            pub use ::#device::Peripherals;

            #(#mod_items)*
        }
    });

    main.push(quote! {
        let init: fn(#(#tys),*) = #init;

        init(#(#exprs),*);

        unsafe {
            #krate::enable();
        }
    });
}

fn resources(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    let mut items = vec![];
    let mut impls = vec![];

    for (name, ownership) in ownerships {
        let _name = Ident::new(format!("_{}", name.as_ref()));
        let mut impl_items = vec![];

        if ownership.is_owned() {
            if let Some(resource) = app.resources.get(name) {
                // For owned resources we don't need borrow(), just get()
                let expr = &resource.expr;
                let ty = &resource.ty;

                root.push(quote! {
                    static mut #_name: #ty = #expr;
                });
            } else {
                // Peripheral
                continue;
            }
        } else {
            if let Some(resource) = app.resources.get(name) {
                let expr = &resource.expr;
                let ty = &resource.ty;

                root.push(quote! {
                    static mut #_name: #ty = #expr;
                });

                impl_items.push(quote! {
                    type Data = #ty;

                    fn borrow<'cs>(
                        &'cs self,
                        t: &'cs #krate::Threshold,
                    ) -> &'cs #krate::Static<#ty> {
                        assert!(t.value() > 0);

                        unsafe { #krate::Static::ref_(&#_name) }
                    }

                    fn borrow_mut<'cs>(
                        &'cs mut self,
                        t: &'cs #krate::Threshold,
                    ) -> &'cs mut #krate::Static<#ty> {
                        assert!(t.value() > 0);

                        unsafe { #krate::Static::ref_mut(&mut #_name) }
                    }

                    fn claim<R, F>(
                        &self,
                        t: &mut #krate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &#krate::Static<#ty>,
                            &mut #krate::Threshold) -> R
                    {
                        unsafe {
                            #krate::claim(
                                #krate::Static::ref_(&#_name),
                                t,
                                f,
                            )
                        }
                    }

                    fn claim_mut<R, F>(
                        &mut self,
                        t: &mut #krate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &mut #krate::Static<#ty>,
                            &mut #krate::Threshold) -> R
                    {
                        unsafe {
                            #krate::claim(
                                #krate::Static::ref_mut(&mut #_name),
                                t,
                                f,
                            )
                        }
                    }
                });
            } else {
                // Peripheral
                impl_items.push(quote! {
                    type Data = #device::#name;

                    fn borrow<'cs>(
                        &'cs self,
                        t: &'cs #krate::Threshold,
                    ) -> &'cs #krate::Static<#device::#name> {
                        assert!(t.value() > 0);

                        unsafe { #krate::Static::ref_(&*#device::#name.get()) }
                    }

                    fn borrow_mut<'cs>(
                        &'cs mut self,
                        t: &'cs #krate::Threshold,
                    ) -> &'cs mut #krate::Static<#device::#name> {
                        assert!(t.value() > 0);

                        unsafe {
                            #krate::Static::ref_mut(&mut *#device::#name.get())
                        }
                    }

                    fn claim<R, F>(
                        &self,
                        t: &mut #krate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &#krate::Static<#device::#name>,
                            &mut #krate::Threshold) -> R
                    {
                        unsafe {
                            #krate::claim(
                                #krate::Static::ref_(&*#device::#name.get()),
                                t,
                                f,
                            )
                        }
                    }

                    fn claim_mut<R, F>(
                        &mut self,
                        t: &mut #krate::Threshold,
                        f: F,
                    ) -> R
                    where
                        F: FnOnce(
                            &mut #krate::Static<#device::#name>,
                            &mut #krate::Threshold) -> R
                    {
                        unsafe {
                            #krate::claim(
                                #krate::Static::ref_mut(
                                    &mut *#device::#name.get(),
                                ),
                                t,
                                f,
                            )
                        }
                    }
                });
            }

            impls.push(quote! {
                #[allow(unsafe_code)]
                unsafe impl #krate::Resource for _resource::#name {
                    #(#impl_items)*
                }
            });

            items.push(quote! {
                #[allow(non_camel_case_types)]
                pub struct #name { _0: () }

                #[allow(unsafe_code)]
                impl #name {
                    pub unsafe fn new() -> Self {
                        #name { _0: () }
                    }
                }
            });
        }
    }

    if !items.is_empty() {
        root.push(quote! {
            mod _resource {
                #(#items)*
            }
        });
    }

    root.push(quote! {
        #(#impls)*
    });
}

fn tasks(app: &App, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    for (name, task) in &app.tasks {
        let mut exprs = vec![];
        let mut fields = vec![];
        let mut items = vec![];

        let has_resources = !task.resources.is_empty();

        let mut needs_reexport = false;
        let lifetime = if has_resources {
            Some(quote!('a))
        } else {
            None
        };

        if has_resources {
            for name in &task.resources {
                let _name = Ident::new(format!("_{}", name.as_ref()));

                if let Some(resource) = app.resources.get(name) {
                    needs_reexport = true;
                    let ty = &resource.ty;

                    fields.push(quote! {
                        pub #name: &'a mut ::#krate::Static<#ty>,
                    });

                    exprs.push(quote! {
                        #name: ::#krate::Static::ref_mut(
                            &mut super::#_name,
                        ),
                    });
                } else {
                    fields.push(quote! {
                        pub #name: &'a mut #krate::Static<::#device::#name>,
                    });

                    exprs.push(quote! {
                        #name: #krate::Static::ref_mut(
                            &mut *::#device::#name.get(),
                        ),
                    });
                }
            }

            if needs_reexport {
                let rname = Ident::new(format!("_{}Resources", name));
                root.push(quote! {
                    #[allow(non_camel_case_types)]
                    #[allow(non_snake_case)]
                    pub struct #rname<#lifetime> {
                        #(#fields)*
                    }
                });

                items.push(quote! {
                    pub use ::#rname as Resources;
                });
            } else {
                items.push(quote! {
                    #[allow(non_snake_case)]
                    pub struct Resources<#lifetime> {
                        #(#fields)*
                    }
                });
            }

            items.push(quote! {
                #[allow(unsafe_code)]
                impl<#lifetime> Resources<#lifetime> {
                    pub unsafe fn new() -> Self {
                        Resources {
                            #(#exprs)*
                        }
                    }
                }
            });
        }

        if let Some(path) = task.path.as_ref() {
            let mut tys = vec![];
            let mut exprs = vec![];

            if has_resources {
                tys.push(quote!(&mut #krate::Threshold));
                tys.push(quote!(#name::Resources));

                exprs.push(quote!(&mut #krate::Threshold::max()));
                exprs.push(quote!(#name::Resources::new()));
            }

            let _name = Ident::new(format!("_{}", name));
            let export_name =
                Lit::Str(name.as_ref().to_owned(), StrStyle::Cooked);
            root.push(quote! {
                #[allow(non_snake_case)]
                #[allow(unsafe_code)]
                #[export_name = #export_name]
                pub unsafe extern "msp430-interrupt" fn #_name() {
                    let f: fn(#(#tys,)*) = #path;

                    f(#(#exprs,)*)
                }
            });
        }

        root.push(quote! {
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            #[allow(unsafe_code)]
            mod #name {
                #(#items)*
            }
        });
    }
}
