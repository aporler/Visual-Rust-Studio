//! Traductions intégrées à l'exécutable (répertoire `locales/` du dépôt).
//!
//! Pour ajouter une langue intégrée : créer `locales/<code>/main.ftl` puis ajouter
//! une ligne ci-dessous. Les langues externes n'ont pas besoin de cette étape.

macro_rules! ftl {
    ($lang:literal, $file:literal) => {
        (
            $lang,
            $file,
            include_str!(concat!("../../../locales/", $lang, "/", $file)),
        )
    };
}

/// (langue, fichier, contenu)
pub(crate) const EMBEDDED: &[(&str, &str, &str)] = &[
    ftl!("en-US", "main.ftl"),
    ftl!("en-US", "designer.ftl"),
    ftl!("en-US", "errors.ftl"),
    ftl!("en-US", "editor.ftl"),
    ftl!("en-US", "examples.ftl"),
    ftl!("en-CA", "main.ftl"),
    ftl!("fr-CA", "main.ftl"),
    ftl!("fr-CA", "designer.ftl"),
    ftl!("fr-CA", "errors.ftl"),
    ftl!("fr-CA", "editor.ftl"),
    ftl!("fr-CA", "examples.ftl"),
    ftl!("fr-FR", "main.ftl"),
    ftl!("es-ES", "main.ftl"),
];
