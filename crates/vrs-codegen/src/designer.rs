//! Génération du fichier `<fenêtre>_designer.rs` et de `forms/mod.rs`.

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use vrs_components::{ComponentKind, Control, EventKind, Feature, FormDocument};

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
    let mut extra = Vec::new();
    if c.kind.has(Feature::Checked) {
        let checked = c.checked;
        extra.push(quote!(checked: #checked,));
    }
    if c.kind.has(Feature::Group) {
        let group = string_expr(&c.group);
        extra.push(quote!(group: #group,));
    }
    if c.kind.has(Feature::Items) {
        let items = c.items.iter().map(|i| quote!(String::from(#i)));
        let selected = match c.selected.filter(|i| *i < c.items.len()) {
            Some(i) => quote!(Some(#i)),
            None => quote!(None),
        };
        extra.push(quote!(items: vec![#(#items),*], selected: #selected,));
    }
    if c.kind.has(Feature::Value) {
        let (v, min, max) = (f32_lit(c.value), f32_lit(c.minimum), f32_lit(c.maximum));
        extra.push(quote!(value: #v, minimum: #min, maximum: #max,));
    }
    let color = |c: Option<[u8; 4]>| match c {
        Some([r, g, b, 255]) => quote!(Some(vrs::Color::from_rgb(#r, #g, #b))),
        Some([r, g, b, a]) => quote!(Some(vrs::Color::from_rgba_unmultiplied(#r, #g, #b, #a))),
        None => quote!(None),
    };
    let back = color(c.back_color);
    let fore = color(c.fore_color);
    let font = match c.font_size {
        Some(s) => {
            let s = f32_lit(s);
            quote!(Some(#s))
        }
        None => quote!(None),
    };
    let a = c.anchor;
    let anchor = if a == vrs_components::Anchor::default() {
        quote!(vrs::Anchor::TOP_LEFT)
    } else {
        let (l, t, r, b) = (a.left, a.top, a.right, a.bottom);
        quote!(vrs::Anchor::new(#l, #t, #r, #b))
    };
    quote! {
        vrs::#ty {
            text: #text,
            bounds: vrs::Bounds::new(#x, #y, #w, #h),
            visible: #visible,
            enabled: #enabled,
            tooltip: #tooltip,
            back_color: #back,
            fore_color: #fore,
            font_size: #font,
            anchor: #anchor,
            #(#extra)*
        }
    }
}

fn event_condition(event: EventKind) -> Option<TokenStream> {
    match event {
        EventKind::Click => Some(quote!(clicked())),
        EventKind::TextChanged
        | EventKind::CheckedChanged
        | EventKind::SelectionChanged
        | EventKind::ValueChanged => Some(quote!(changed())),
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
    // Les cadres (GroupBox, Panel) sont dessinés en premier, derrière les autres.
    let mut ordered: Vec<&Control> = form.controls.iter().collect();
    ordered.sort_by_key(|c| !c.kind.is_container());
    let mut exclusive = Vec::new();
    for c in ordered {
        let name = ident(&c.name);
        let name_str = c.name.as_str();
        let render = ident(c.kind.render_fn());
        let arg = if c.kind.render_mut() {
            quote!(&mut self.#name)
        } else {
            quote!(&self.#name)
        };
        let handlers: Vec<_> = c
            .events
            .iter()
            .filter_map(|(e, h)| event_condition(*e).map(|cond| (cond, ident(h))))
            .collect();
        // Boutons d'option : cocher l'un décoche les autres du même groupe.
        let others: Vec<Ident> = if c.kind == ComponentKind::RadioButton {
            form.controls
                .iter()
                .filter(|o| {
                    o.kind == ComponentKind::RadioButton && o.group == c.group && o.name != c.name
                })
                .map(|o| ident(&o.name))
                .collect()
        } else {
            Vec::new()
        };
        if !others.is_empty() {
            exclusive.push(quote! {
                if #name.changed() {
                    #(self.#others.checked = false;)*
                }
            });
        }
        if handlers.is_empty() && others.is_empty() {
            draws.push(quote!(let _ = vrs::#render(ui, #name_str, #arg);));
        } else {
            uses_app |= !handlers.is_empty();
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
    let layout_call = (!form.controls.is_empty()).then(|| quote!(vrs::layout(ui, [#w, #h]);));
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
                #layout_call
                #(#draws)*
                #(#exclusive)*
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
    fn new_components_and_radio_groups() {
        let mut f = FormDocument::new("Form1");
        f.add_control(ComponentKind::GroupBox, 0.0, 0.0);
        let r1 = f.add_control(ComponentKind::RadioButton, 8.0, 8.0);
        f.add_control(ComponentKind::RadioButton, 8.0, 32.0);
        let cb = f.add_control(ComponentKind::ComboBox, 8.0, 60.0);
        f.add_control(ComponentKind::Slider, 8.0, 90.0);
        f.add_control(ComponentKind::ProgressBar, 8.0, 120.0);
        f.control_mut(&r1).unwrap().checked = true;
        let c = f.control_mut(&cb).unwrap();
        c.items = vec!["Un".into(), "Deux".into()];
        c.selected = Some(1);
        c.events.insert(
            EventKind::SelectionChanged,
            "combo_box1_selection_changed".into(),
        );
        let src = generate_designer(&f);
        syn::parse_file(&src).unwrap();
        assert!(src.contains("items: vec![String::from(\"Un\"), String::from(\"Deux\")]"));
        assert!(src.contains("selected: Some(1usize)") || src.contains("selected: Some(1)"));
        assert!(src.contains("self.radio_button2.checked = false;"));
        assert!(src.contains("if combo_box1.changed()"));
        assert!(src.contains("vrs::slider(ui, \"slider1\", &mut self.slider1)"));
        assert!(src.contains("vrs::progress_bar(ui, \"progress_bar1\", &self.progress_bar1)"));
        // Le cadre est dessiné en premier.
        assert!(src.find("vrs::group_box").unwrap() < src.find("vrs::radio_button").unwrap());
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
