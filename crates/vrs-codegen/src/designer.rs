//! Génération du fichier `<fenêtre>_designer.rs` et de `forms/mod.rs`.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use vrs_components::{ComponentKind, Control, EventKind, FormDocument};

fn ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

fn f32_lit(v: f32) -> Literal {
    // Arrondi au centième pour un code lisible (les positions viennent de la souris).
    Literal::f32_unsuffixed((v * 100.0).round() / 100.0)
}

fn string_expr(s: &str) -> TokenStream {
    if s.is_empty() {
        quote!(String::new())
    } else {
        quote!(String::from(#s))
    }
}

fn state_init(c: &Control) -> TokenStream {
    let ty = ident(c.kind.state_type());
    let text = string_expr(&c.text);
    let tooltip = string_expr(&c.tooltip);
    let (x, y, w, h) = (
        f32_lit(c.x),
        f32_lit(c.y),
        f32_lit(c.width),
        f32_lit(c.height),
    );
    let visible = c.visible;
    let enabled = c.enabled;
    quote! {
        vrs::#ty {
            text: #text,
            bounds: vrs::Bounds::new(#x, #y, #w, #h),
            visible: #visible,
            enabled: #enabled,
            tooltip: #tooltip,
        }
    }
}

fn event_condition(event: EventKind) -> Option<TokenStream> {
    match event {
        EventKind::Click => Some(quote!(clicked())),
        EventKind::TextChanged => Some(quote!(changed())),
        EventKind::Load | EventKind::Closing | EventKind::Closed => None,
    }
}

/// Génère le fichier du concepteur d'une fenêtre (déjà formaté, avec en-tête).
pub fn generate_designer(form: &FormDocument) -> String {
    let form_ty = ident(&form.name);
    let title = &form.title;
    let (w, h) = (f32_lit(form.width), f32_lit(form.height));

    let fields = form.controls.iter().map(|c| {
        let name = ident(&c.name);
        let ty = ident(c.kind.state_type());
        let doc = format!(" {} `{}`.", c.kind.type_name(), c.name);
        quote! {
            #[doc = #doc]
            pub #name: vrs::#ty
        }
    });
    let inits = form.controls.iter().map(|c| {
        let name = ident(&c.name);
        let init = state_init(c);
        quote!(#name: #init)
    });

    let mut uses_app = false;
    let mut draws = Vec::new();
    let mut dispatch = Vec::new();
    for c in &form.controls {
        let name = ident(&c.name);
        let name_str = c.name.as_str();
        let render = ident(c.kind.render_fn());
        let arg = match c.kind {
            ComponentKind::TextBox => quote!(&mut self.#name),
            ComponentKind::Button | ComponentKind::Label => quote!(&self.#name),
        };
        let handlers: Vec<_> = c
            .events
            .iter()
            .filter_map(|(e, h)| event_condition(*e).map(|cond| (cond, ident(h))))
            .collect();
        if handlers.is_empty() {
            draws.push(quote!(let _ = vrs::#render(ui, #name_str, #arg);));
        } else {
            uses_app = true;
            draws.push(quote!(let #name = vrs::#render(ui, #name_str, #arg);));
            for (cond, handler) in handlers {
                dispatch.push(quote! {
                    if #name.#cond {
                        self.#handler(app);
                    }
                });
            }
        }
    }
    let app_param = if uses_app {
        ident("app")
    } else {
        ident("_app")
    };
    let ui_param = if form.controls.is_empty() {
        ident("_ui")
    } else {
        ident("ui")
    };

    let load_fn = form.events.get(&EventKind::Load).map(|h| {
        let h = ident(h);
        quote! {
            fn load(&mut self, app: &mut AppContext) {
                self.#h(app);
            }
        }
    });
    let closing_fn = form.events.get(&EventKind::Closing).map(|h| {
        let h = ident(h);
        quote! {
            fn closing(&mut self, app: &mut AppContext) -> bool {
                self.#h(app)
            }
        }
    });
    let closed_fn = form.events.get(&EventKind::Closed).map(|h| {
        let h = ident(h);
        quote! {
            fn closed(&mut self, app: &mut AppContext) {
                self.#h(app);
            }
        }
    });

    let struct_doc = format!(
        " Fenêtre `{}` : un champ par contrôle placé dans le concepteur.",
        form.name
    );
    let show_doc = " Dessine les contrôles, puis appelle les gestionnaires d'événements \
                    (après le dessin, pour éviter les conflits d'emprunt).";

    let file: TokenStream = quote! {
        use crate::vrs::{self, AppContext, egui};

        #[doc = #struct_doc]
        pub struct #form_ty {
            #(#fields,)*
        }

        impl Default for #form_ty {
            fn default() -> Self {
                Self {
                    #(#inits,)*
                }
            }
        }

        impl vrs::Form for #form_ty {
            fn title(&self) -> String {
                String::from(#title)
            }

            fn size(&self) -> [f32; 2] {
                [#w, #h]
            }

            #[doc = #show_doc]
            fn show(&mut self, #ui_param: &mut egui::Ui, #app_param: &mut AppContext) {
                #(#draws)*
                #(#dispatch)*
            }

            #load_fn
            #closing_fn
            #closed_fn
        }
    };
    let parsed: syn::File = syn::parse2(file).expect("le générateur produit du Rust valide");
    let body = prettyplease::unparse(&parsed);
    let source = format!("forms/{}.{}", form.name, vrs_components::FORM_EXTENSION);
    format!(
        "{}#![allow(dead_code)]\n\n{body}",
        super::generated_header(&source)
    )
}

/// Génère `src/forms/mod.rs`.
pub fn generate_forms_mod(forms: &[FormDocument], startup: &str, last_form_closes: bool) -> String {
    let mut mods = Vec::new();
    let mut uses = Vec::new();
    for f in forms {
        let m = ident(&f.module_name());
        let d = ident(&format!("{}_designer", f.module_name()));
        let ty = ident(&f.name);
        mods.push(quote!(mod #m; mod #d;));
        uses.push(quote!(#[allow(unused_imports)] pub use #d::#ty;));
    }
    let startup_fn = forms.iter().find(|f| f.name == startup).map(|f| {
        let ty = ident(&f.name);
        quote! {
            /// Fenêtre affichée au démarrage de l'application.
            pub fn startup_form() -> #ty {
                #ty::default()
            }
        }
    });
    let strategy = if last_form_closes {
        quote!(LastFormCloses)
    } else {
        quote!(MainFormCloses)
    };
    let file: TokenStream = quote! {
        #(#mods)*
        #(#uses)*
        #startup_fn
        /// Comportement à la fermeture de la fenêtre principale.
        pub const CLOSE_STRATEGY: crate::vrs::CloseStrategy = crate::vrs::CloseStrategy::#strategy;
    };
    let parsed: syn::File = syn::parse2(file).expect("mod.rs valide");
    format!(
        "{}{}",
        super::generated_header("liste des fenêtres du projet (.vrsproj)"),
        prettyplease::unparse(&parsed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FormDocument {
        let mut f = FormDocument::new("Form1");
        f.title = "Démo \"VRS\"".into();
        let b = f.add_control(ComponentKind::Button, 16.0, 16.0);
        f.add_control(ComponentKind::TextBox, 16.0, 56.0);
        f.add_control(ComponentKind::Label, 16.0, 96.123);
        f.control_mut(&b)
            .unwrap()
            .events
            .insert(EventKind::Click, "button1_click".into());
        f
    }

    #[test]
    fn designer_contains_fields_and_dispatch() {
        let src = generate_designer(&sample());
        assert!(src.starts_with("// ----"));
        assert!(src.contains("pub struct Form1"));
        assert!(src.contains("pub button1: vrs::ButtonState"));
        assert!(src.contains("pub text_box1: vrs::TextBoxState"));
        assert!(src.contains("let button1 = vrs::button(ui, \"button1\", &self.button1);"));
        assert!(src.contains("vrs::text_box(ui, \"text_box1\", &mut self.text_box1)"));
        assert!(src.contains("self.button1_click(app);"));
        assert!(src.contains("96.12"));
        assert!(src.contains(r#"String::from("Démo \"VRS\"")"#));
        // Le résultat est du Rust valide.
        syn::parse_file(&src).unwrap();
    }

    #[test]
    fn no_events_means_unused_app_param() {
        let mut f = FormDocument::new("Form2");
        f.add_control(ComponentKind::Button, 0.0, 0.0);
        let src = generate_designer(&f);
        assert!(src.contains("_app: &mut AppContext"));
        assert!(src.contains("let _ = vrs::button"));
        // Fenêtre vide : aucun paramètre inutilisé (pas d'avertissement).
        let empty = generate_designer(&FormDocument::new("Form3"));
        assert!(empty.contains("_ui: &mut egui::Ui, _app: &mut AppContext"));
    }

    #[test]
    fn deterministic() {
        assert_eq!(generate_designer(&sample()), generate_designer(&sample()));
    }

    #[test]
    fn forms_mod() {
        let src = generate_forms_mod(&[sample(), FormDocument::new("Form2")], "Form1", false);
        assert!(src.contains("CloseStrategy::MainFormCloses"));
        assert!(src.contains("mod form1;"));
        assert!(src.contains("mod form2_designer;"));
        assert!(src.contains("pub use form1_designer::Form1;"));
        assert!(src.contains("pub fn startup_form() -> Form1"));
        syn::parse_file(&src).unwrap();
    }
}
