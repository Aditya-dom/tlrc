use std::env;
use std::iter;
use std::mem;

/// Prints a warning.
macro_rules! warnln {
    ( $( $arg:tt )* ) => {
        if !$crate::QUIET.load(std::sync::atomic::Ordering::Relaxed) {
            use std::io::Write;
            let mut stderr = std::io::stderr().lock();
            write!(stderr, "{} ", yansi::Paint::new("warning:").fg(yansi::Color::Yellow).bold())?;
            writeln!(stderr, $($arg)*)?;
        }
    };
}

/// Prints a status message.
macro_rules! infoln {
    ( $( $arg:tt )* ) => {
        if !$crate::QUIET.load(std::sync::atomic::Ordering::Relaxed) {
            use std::io::Write;
            let mut stderr = std::io::stderr().lock();
            write!(stderr, "{} ", yansi::Paint::new("info:").fg(yansi::Color::Cyan).bold())?;
            writeln!(stderr, $($arg)*)?;
        }
    };
}

pub(crate) use {infoln, warnln};

pub fn get_languages_from_env() -> Vec<String> {
    // https://github.com/tldr-pages/tldr/blob/main/CLIENT-SPECIFICATION.md#language

    let var_lang = env::var("LANG").ok();
    let var_language = env::var("LANGUAGE").ok();

    if var_lang.is_none() {
        return vec!["en".to_string()];
    }

    let var_lang = var_lang.unwrap();
    let var_language = var_language.as_deref();

    let mut result = vec![];
    let languages = var_language
        .unwrap_or("")
        .split(':')
        .chain(iter::once(&*var_lang));

    for lang in languages {
        if lang.len() >= 5 && lang.chars().nth(2) == Some('_') {
            // <language>_<country> (ll_CC - 5 characters)
            result.push(&lang[..5]);
            // <language> (ll - 2 characters)
            result.push(&lang[..2]);
        } else if lang.len() == 2 {
            result.push(lang);
        }
    }

    result.push("en");
    result.dedup_nosort();

    result.into_iter().map(String::from).collect()
}

/// Convert language codes to directory names in the cache.
pub fn languages_to_langdirs(languages: &[String]) -> Vec<String> {
    languages
        .iter()
        .map(|lang| {
            if lang == "en" {
                "pages".to_string()
            } else {
                format!("pages.{lang}")
            }
        })
        .collect()
}

trait Dedup {
    /// Deduplicate a vector in place preserving the order of elements.
    fn dedup_nosort(&mut self);
}

impl<T> Dedup for Vec<T>
where
    T: PartialEq,
{
    fn dedup_nosort(&mut self) {
        let old = mem::replace(self, Vec::with_capacity(self.len()));
        for x in old {
            if !self.contains(&x) {
                self.push(x);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn prepare_env(lang: Option<&str>, language: Option<&str>) {
        if let Some(lang) = lang {
            env::set_var("LANG", lang);
        } else {
            env::remove_var("LANG");
        }

        if let Some(language) = language {
            env::set_var("LANGUAGE", language);
        } else {
            env::remove_var("LANGUAGE");
        }
    }

    #[test]
    fn env_languages() {
        prepare_env(Some("cz"), Some("it:cz:de"));
        assert_eq!(get_languages_from_env(), ["it", "cz", "de", "en"]);

        prepare_env(Some("cz"), Some("it:de:fr"));
        assert_eq!(get_languages_from_env(), ["it", "de", "fr", "cz", "en"]);

        prepare_env(Some("it"), None);
        assert_eq!(get_languages_from_env(), ["it", "en"]);

        prepare_env(None, Some("it:cz"));
        assert_eq!(get_languages_from_env(), ["en"]);

        prepare_env(None, None);
        assert_eq!(get_languages_from_env(), ["en"]);

        prepare_env(Some("en_US.UTF-8"), Some("de_DE.UTF-8:pl:en"));
        assert_eq!(
            get_languages_from_env(),
            ["de_DE", "de", "pl", "en", "en_US"]
        );
    }
}
