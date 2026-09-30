use once_cell::sync::Lazy;
use regex::Regex;

/// Pre-compiled regex cache for all detectors.
/// Patterns are compiled once at startup instead of per-line per-pattern.
pub struct RegexCache {
    // slop_detector patterns
    pub slop_manual_loop: Regex,
    pub slop_clone: Regex,
    pub slop_unwrap: Regex,
    pub slop_panic: Regex,
    pub slop_index: Regex,
    pub slop_single_impl: Regex,
    pub slop_config_const: Regex,
    pub slop_factory_single: Regex,
    // yagni_detector patterns
    pub yagni_function: Regex,
    pub yagni_variable: Regex,
    // over_engineer_detector patterns
    pub over_single_use: Regex,
    pub over_manual_sort: Regex,
    pub over_manual_join: Regex,
    pub over_manual_filter: Regex,
    pub over_dead_code_fn: Regex,
    pub over_vec_new_for: Regex,
    // minimal_check_detector patterns
    pub check_fn: Regex,
    pub check_assert: Regex,
    // frontend_detector patterns
    pub front_img: Regex,
    // api_security_detector patterns (none - uses string matching)
    // event_behavior_detector patterns (none - uses serde_json)
    // string stripping
    pub string_literal: Regex,
}

impl RegexCache {
    pub fn global() -> &'static RegexCache {
        static CACHE: Lazy<RegexCache> = Lazy::new(RegexCache::new);
        &CACHE
    }

    fn new() -> Self {
        let r = |pat: &str| Regex::new(pat).expect("built-in regex");
        Self {
            slop_manual_loop: r(r"for\s+\w+\s+in\s+0\s*\.\.\s*\w+\.len\(\)"),
            slop_clone: r(r"\.clone\(\)"),
            slop_unwrap: r(r"\.unwrap\(\)"),
            slop_panic: r(r"\b(?:panic|todo|unimplemented)!\s*\("),
            slop_index: r(r"\b\w+\s*\[[^\]]+\]"),
            slop_single_impl: r(r"(?:pub\s+)?trait\s+\w+"),
            slop_config_const: r(r"(?:static|const)\s+\w+.*=.*;"),
            slop_factory_single: r(r"fn\s+(?:create|new|build|make)_\w+"),
            yagni_function: r(r"^\s*(pub\s+)?fn\s+([A-Za-z_]\w*)\s*\("),
            yagni_variable: r(r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*="),
            over_single_use: r(r"(?:pub\s+)?(?:struct|trait|enum)\s+(\w+)"),
            over_manual_sort: r(r"\.sort_by\(|.*\|.*partial_cmp"),
            over_manual_join: r(r#"\.join\(\s*""\s*\)"#),
            over_manual_filter: r(r"\.split\(.*\)\.filter\(.*\)\.collect"),
            over_dead_code_fn: r(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_]\w*)\s*\("),
            over_vec_new_for: r(r"Vec::new\(\)"),
            check_fn: r(r"^\s*(?:pub\s+)?(?:fn|async\s+fn)\s+([A-Za-z_]\w*)\s*[\(<]"),
            check_assert: r(r"\b(?:assert|debug_assert|assert_eq|assert_ne)\b"),
            front_img: r(r"(?is)<img\b[^>]*>"),
            string_literal: r(r#""([^"\\]|\\.)*""#),
        }
    }
}

/// Accessor for the global regex cache.
pub fn cache() -> &'static RegexCache {
    RegexCache::global()
}