use regex::{escape, Regex};

fn pattern_from_glob(glob: &str) -> String {
    let mut prefix = String::new();
    let mut pattern = glob.chars().fold(String::from('^'), |mut acc, c| {
        // TODO: Check if escaping glob characters is supported
        match c {
            '?' => {
                if !prefix.is_empty() {
                    acc.extend(escape(&prefix).drain(..));
                    prefix.clear();
                }
                acc.push('.');
            }
            '*' => {
                if !prefix.is_empty() {
                    acc.extend(escape(&prefix).drain(..));
                    prefix.clear();
                }
                acc.push_str(".*");
            }
            _ => {
                prefix.push(c);
            }
        };
        acc
    });
    pattern.extend(escape(&prefix).drain(..));
    pattern.push('$');
    pattern
}

pub fn try_regex_from_glob(glob: &str) -> Result<Regex, regex::Error> {
    Regex::new(&pattern_from_glob(glob))
}

pub fn glob_matches(glob: &str, target: &str) -> bool {
    if !glob.contains(['?', '*']) {
        return target == glob;
    }
    // TODO: Errors should be impossible.
    try_regex_from_glob(glob).map_or(false, |re| re.is_match(target))
}

#[cfg(test)]
mod test {
    use super::{glob_matches, pattern_from_glob};

    #[test]
    fn test_literal_pattern() {
        let pattern = pattern_from_glob("");
        assert_eq!(pattern, "^$");
        let pattern = pattern_from_glob("test");
        assert_eq!(pattern, "^test$");
    }

    #[test]
    fn test_single_wildcard_pattern() {
        let pattern = pattern_from_glob("?");
        assert_eq!(pattern, "^.$");
        let pattern = pattern_from_glob("a?");
        assert_eq!(pattern, "^a.$");
        let pattern = pattern_from_glob("?b");
        assert_eq!(pattern, "^.b$");
        let pattern = pattern_from_glob("a?c");
        assert_eq!(pattern, "^a.c$");
    }

    #[test]
    fn test_multi_wildcard_pattern() {
        let pattern = pattern_from_glob("*");
        assert_eq!(pattern, "^.*$");
        let pattern = pattern_from_glob("a*");
        assert_eq!(pattern, "^a.*$");
        let pattern = pattern_from_glob("*b");
        assert_eq!(pattern, "^.*b$");
        let pattern = pattern_from_glob("a*c");
        assert_eq!(pattern, "^a.*c$");
    }

    #[test]
    fn test_literal() {
        assert!(glob_matches("", ""));
        assert!(glob_matches("sometext", "sometext"));
        assert!(! glob_matches("", "sometext"));
        assert!(! glob_matches("sometext", ""));
        assert!(! glob_matches("sometext", "sometextandmore"));
    }

    #[test]
    fn test_single_wildcard() {
        assert!(glob_matches("a?c", "abc"));
        assert!(! glob_matches("a?c", "ac"));
        assert!(! glob_matches("a?c", "abbc"));
    }

    #[test]
    fn test_multi_wildcard() {
        assert!(glob_matches("a*c", "ac"));
        assert!(glob_matches("a*c", "abc"));
        assert!(glob_matches("a*c", "abbc"));
        assert!(! glob_matches("a*c", "bc"));
        assert!(! glob_matches("a*c", "ab"));
    }
}
