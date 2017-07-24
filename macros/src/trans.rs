use quote::{Ident, Tokens};

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
        fn main() {
            #(#main)*
        }
    });

    quote!(#(#root)*)
}

// Check that the interrupts are valid
// Sadly we can't do this test at expansion time. Instead we'll generate some
// code that won't compile if the interrupt name is invalid.
fn check(app: &App, main: &mut Vec<Tokens>) {
    let device = &app.device;

    for task in app.tasks.keys() {
        main.push(quote! {
            let _ = #device::Interrupt::#task;
        });
    }
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
            let ty = &resource.ty;

            fields.push(quote! {
                pub #name: &'a mut #krate::Static<#ty>,
            });

            rexprs.push(quote! {
                #name: ::#krate::Static::ref_mut(&mut *super::#name.get()),
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
        let mut lifetime = None;

        let mut needs_reexport = false;
        for name in &app.idle.resources {
            if ownerships[name].is_owned() {
                // is not a peripheral?
                if app.resources.get(name).is_some() {
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
            if ownerships[name].is_owned() {
                lifetime = Some(quote!('a));
                if let Some(resource) = app.resources.get(name) {
                    let ty = &resource.ty;

                    rfields.push(quote! {
                        pub #name: &'a mut ::#krate::Static<#ty>,
                    });

                    rexprs.push(quote! {
                        #name: ::#krate::Static::ref_mut(
                            &mut *#super_::#name.get(),
                        ),
                    });
                } else {
                    rfields.push(quote! {
                        pub #name: &'a mut ::#device::#name,
                    });

                    rexprs.push(quote! {
                        #name: &mut *::#device::#name.get(),
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
                pub struct _idleResources<#lifetime> {
                    #(#rfields)*
                }
            });

            mod_items.push(quote! {
                pub use ::_idleResources as Resources;
            });
        } else {
            mod_items.push(quote! {
                #[allow(non_snake_case)]
                pub struct Resources<#lifetime> {
                    #(#rfields)*
                }
            });
        }

        mod_items.push(quote! {
            impl<#lifetime> Resources<#lifetime> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#rexprs)*
                    }
                }
            }
        });

        tys.push(quote!(idle::Resources));
        exprs.push(quote!(unsafe { idle::Resources::new() }));
    }

    root.push(quote! {
        mod idle {
            #(#mod_items)*
        }
    });

    let idle = &app.idle.path;
    main.push(quote! {
        // type check
        let idle: fn(#(#tys),*) -> ! = #idle;

        idle(#(#exprs),*);
    });
}

fn resources(app: &App, ownerships: &Ownerships, root: &mut Vec<Tokens>) {
    let device = &app.device;
    let krate = krate();

    let mut items = vec![];
    let mut impls = vec![];

    for (name, ownership) in ownerships {
        let mut impl_items = vec![];

        if ownership.is_owned() {
            if let Some(resource) = app.resources.get(name) {
                // For owned resources we don't need borrow(), just get()
                let expr = &resource.expr;
                let ty = &resource.ty;

                root.push(quote! {
                    static #name: #krate::Resource<#ty> =
                        #krate::Resource::new(#expr);
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
                    static #name: #krate::Resource<#ty> =
                        #krate::Resource::new(#expr);
                });

                impl_items.push(quote! {
                    pub fn borrow<'cs>(
                        &'cs self,
                        cs: &'cs #krate::CriticalSection,
                    ) -> &'cs #krate::Static<#ty> {
                        #name.borrow(cs)
                    }

                    pub fn borrow_mut<'cs>(
                        &'cs mut self,
                        cs: &'cs #krate::CriticalSection,
                    ) -> &'cs mut #krate::Static<#ty> {
                        unsafe { #name.borrow_mut(cs)}
                    }
                });
            } else {
                // Peripheral
                impl_items.push(quote! {
                    pub fn borrow<'cs>(
                        &'cs self,
                        cs: &'cs #krate::CriticalSection,
                    ) -> &'cs #device::#name {
                        unsafe { &*#device::#name.get() }
                    }
                });
            }

            impls.push(quote! {
                #[allow(dead_code)]
                impl _resource::#name {
                    #(#impl_items)*
                }
            });

            items.push(quote! {
                #[allow(non_camel_case_types)]
                pub struct #name { _0: () }

                impl #name {
                    pub unsafe fn new() -> Self {
                        #name { _0: () }
                    }
                }
            });
        }
    }

    root.push(quote! {
        mod _resource {
            #(#items)*
        }

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

        let mut needs_reexport = false;
        let lifetime = if task.resources.is_empty() {
            None
        } else {
            Some(quote!('a))
        };
        for name in &task.resources {
            if let Some(resource) = app.resources.get(name) {
                needs_reexport = true;
                let ty = &resource.ty;

                fields.push(quote! {
                    pub #name: &'a mut ::#krate::Static<#ty>,
                });

                exprs.push(quote! {
                    #name: ::#krate::Static::ref_mut(
                        &mut *super::#name.get(),
                    ),
                });
            } else {
                fields.push(quote! {
                    pub #name: &'a mut ::#device::#name,
                });

                exprs.push(quote! {
                    #name: &mut *::#device::#name.get(),
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
            impl<#lifetime> Resources<#lifetime> {
                pub unsafe fn new() -> Self {
                    Resources {
                        #(#exprs)*
                    }
                }
            }
        });

        root.push(quote! {
            #[allow(dead_code)]
            #[allow(non_snake_case)]
            mod #name {
                #(#items)*
            }
        });
    }
}
