//! File matching and secret redaction filters.
//! Filtros para correspondência de arquivos e expurgo de dados sensíveis.

use anyhow::Result;
use regex::Regex;

/// File and directory matcher supporting simple glob patterns (* and ?).
/// Casador de arquivos e diretórios suportando padrões glob simples (* e ?).
#[derive(Debug, Clone)]
pub struct FileMatcher {
    regex: Regex,
}

impl FileMatcher {
    /// Creates a new `FileMatcher` from a glob pattern.
    /// Cria um novo `FileMatcher` a partir de um padrão glob.
    pub fn new(pattern: &str) -> Result<Self> {
        let regex_pattern = format!(
            "^{}$",
            regex::escape(pattern)
                .replace("\\*", ".*")
                .replace("\\?", ".")
        );
        let regex = Regex::new(&regex_pattern)?;
        Ok(Self { regex })
    }

    /// Checks if a file or folder name matches the pattern.
    /// Verifica se o nome do arquivo ou diretório corresponde ao padrão.
    pub fn is_match(&self, name: &str) -> bool {
        self.regex.is_match(name)
    }
}

/// Redactor responsible for replacing sensitive text with `***REDACTED***`.
/// Redator responsável por substituir textos sensíveis por `***REDACTED***`.
#[derive(Debug, Clone)]
pub struct Redactor {
    pattern: Option<Regex>,
}

impl Redactor {
    /// Initializes a `Redactor` with an optional regular expression string.
    /// Inicializa um `Redactor` com uma expressão regular opcional.
    pub fn new(pattern: Option<&str>) -> Result<Self> {
        let pattern = match pattern {
            Some(p) => Some(Regex::new(p)?),
            None => None,
        };
        Ok(Self { pattern })
    }

    /// Redacts all regex pattern matches found in the provided text.
    /// Expurga todas as ocorrências do padrão regex encontradas no texto.
    pub fn redact(&self, text: &str) -> String {
        match &self.pattern {
            Some(re) => re.replace_all(text, "***REDACTED***").to_string(),
            None => text.to_string(),
        }
    }

    /// Returns `true` if an active regex pattern is configured.
    /// Retorna `true` se um padrão regex ativo estiver configurado.
    pub fn has_pattern(&self) -> bool {
        self.pattern.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_matcher_wildcards() {
        let matcher = FileMatcher::new("*.zip").unwrap();
        assert!(matcher.is_match("archive.zip"));
        assert!(matcher.is_match("backup.zip"));
        assert!(!matcher.is_match("archive.tar.gz"));
        assert!(!matcher.is_match("zip.txt"));
    }

    #[test]
    fn test_file_matcher_exact() {
        let matcher = FileMatcher::new("id_rsa").unwrap();
        assert!(matcher.is_match("id_rsa"));
        assert!(!matcher.is_match("id_rsa.pub"));
    }

    #[test]
    fn test_redactor_replacement() {
        let redactor = Redactor::new(Some(r"AKIA[0-9A-Z]{16}")).unwrap();
        let input = "my_key = AKIAIOSFODNN7EXAMPLE;";
        let expected = "my_key = ***REDACTED***;";
        assert_eq!(redactor.redact(input), expected);
    }

    #[test]
    fn test_redactor_no_pattern() {
        let redactor = Redactor::new(None).unwrap();
        let input = "plain_text_value";
        assert_eq!(redactor.redact(input), input);
        assert!(!redactor.has_pattern());
    }
}
