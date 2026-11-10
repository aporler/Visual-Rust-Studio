//! Négociation de la langue et chaîne de repli.

use std::collections::BTreeMap;

use unic_langid::LanguageIdentifier;

/// Règles de repli configurables.
///
/// Pour une langue demandée, la chaîne est construite ainsi :
/// 1. la locale exacte si elle existe (`fr-CA`) ;
/// 2. la variante préférée de la même langue (`fr` → `fr-FR`, `es` → `es-ES`) ;
/// 3. les autres variantes disponibles de la même langue, par ordre alphabétique ;
/// 4. la locale par défaut (`en-US`).
#[derive(Clone, Debug)]
pub struct FallbackPolicy {
    /// Dernier recours (toujours complète).
    pub default: LanguageIdentifier,
    /// Variante préférée par code de langue (`"fr"` → `fr-FR`).
    pub preferred_variant: BTreeMap<String, LanguageIdentifier>,
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        let mut preferred_variant = BTreeMap::new();
        for (lang, loc) in [("fr", "fr-FR"), ("en", "en-US"), ("es", "es-ES")] {
            preferred_variant.insert(lang.to_owned(), loc.parse().expect("locale valide"));
        }
        Self {
            default: super::default_locale(),
            preferred_variant,
        }
    }
}

/// Calcule la chaîne de repli pour `requested` parmi `available`.
///
/// Le résultat n'est jamais vide et se termine toujours par `policy.default`.
pub fn fallback_chain(
    requested: &LanguageIdentifier,
    available: &[LanguageIdentifier],
    policy: &FallbackPolicy,
) -> Vec<LanguageIdentifier> {
    let mut chain: Vec<LanguageIdentifier> = Vec::new();
    let push = |l: &LanguageIdentifier, chain: &mut Vec<LanguageIdentifier>| {
        if !chain.contains(l) {
            chain.push(l.clone());
        }
    };

    if available.contains(requested) {
        push(requested, &mut chain);
    }

    let lang = requested.language.as_str();
    if let Some(pref) = policy.preferred_variant.get(lang)
        && available.contains(pref)
    {
        push(pref, &mut chain);
    }

    let mut same_lang: Vec<&LanguageIdentifier> = available
        .iter()
        .filter(|l| l.language.as_str() == lang)
        .collect();
    same_lang.sort_by_key(|l| l.to_string());
    for l in same_lang {
        push(l, &mut chain);
    }

    push(&policy.default, &mut chain);
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(list: &[&str]) -> Vec<LanguageIdentifier> {
        list.iter().map(|s| s.parse().unwrap()).collect()
    }

    fn chain(req: &str) -> Vec<String> {
        let available = ids(&["en-US", "en-CA", "fr-CA", "fr-FR", "es-ES"]);
        fallback_chain(
            &req.parse().unwrap(),
            &available,
            &FallbackPolicy::default(),
        )
        .iter()
        .map(|l| l.to_string())
        .collect()
    }

    #[test]
    fn exact() {
        assert_eq!(chain("fr-CA"), ["fr-CA", "fr-FR", "en-US"]);
        assert_eq!(chain("en-CA"), ["en-CA", "en-US"]);
    }

    #[test]
    fn same_language() {
        assert_eq!(chain("fr-BE"), ["fr-FR", "fr-CA", "en-US"]);
        assert_eq!(chain("es-MX"), ["es-ES", "en-US"]);
        assert_eq!(chain("fr"), ["fr-FR", "fr-CA", "en-US"]);
    }

    #[test]
    fn unknown_language() {
        assert_eq!(chain("de-DE"), ["en-US"]);
        assert_eq!(chain("ja"), ["en-US"]);
    }
}
