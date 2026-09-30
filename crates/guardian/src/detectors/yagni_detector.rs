use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub call_count: usize,
    pub is_public: bool,
    pub is_test: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableInfo {
    pub name: String,
    pub line: usize,
    pub usage_count: usize,
    pub scope: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YAGNIReport {
    pub unused_functions: HashMap<String, FunctionInfo>,
    pub unused_variables: HashMap<String, VariableInfo>,
    pub unused_imports: HashSet<String>,
    pub over_engineering_score: f64,
}

/// AST visitor to collect function definitions, calls, variable definitions/uses, and imports.
struct YagniVisitor {
    functions: HashMap<String, FunctionInfo>,
    function_calls: HashMap<String, usize>,
    variables: HashMap<String, VariableInfo>,
    variable_uses: HashMap<String, usize>,
    imports: HashSet<String>,
    current_scope: String,
}

impl Default for YagniVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl YagniVisitor {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            function_calls: HashMap::new(),
            variables: HashMap::new(),
            variable_uses: HashMap::new(),
            imports: HashSet::new(),
            current_scope: "root".into(),
        }
    }
}

impl<'ast> Visit<'ast> for YagniVisitor {
    /// Visit function signatures (definitions)
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        let line = 0; // line number requires proc_macro2 "span-locations" feature
        let is_public = matches!(node.vis, syn::Visibility::Public(_));
        let is_test = node.attrs.iter().any(|a| {
            a.path().is_ident("test")
                || a.path()
                    .segments
                    .last()
                    .map_or(false, |s| s.ident == "test")
        });

        self.functions.insert(
            name.clone(),
            FunctionInfo {
                name: name.clone(),
                line,
                call_count: 0,
                is_public,
                is_test,
            },
        );

        visit::visit_item_fn(self, node);
    }

    /// Visit function calls
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(seg) = path.path.segments.last() {
                let name = seg.ident.to_string();
                *self.function_calls.entry(name).or_insert(0) += 1;
            }
        }
        visit::visit_expr_call(self, node);
    }

    /// Visit variable definitions (let bindings)
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let syn::Pat::Ident(pat_ident) = &node.pat {
            let name = pat_ident.ident.to_string();
            let line = 0; // line number requires proc_macro2 "span-locations" feature
            self.variables.insert(
                name.clone(),
                VariableInfo {
                    name: name.clone(),
                    line,
                    usage_count: 0,
                    scope: self.current_scope.clone(),
                },
            );
        }
        visit::visit_local(self, node);
    }

    /// Visit variable uses (identifiers)
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if let Some(seg) = node.path.segments.last() {
            let name = seg.ident.to_string();
            *self.variable_uses.entry(name).or_insert(0) += 1;
        }
        visit::visit_expr_path(self, node);
    }

    /// Visit use statements
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        // Extract the root import name
        let root_name = extract_root_import(&node.tree);
        if !root_name.is_empty() {
            self.imports.insert(root_name.clone());
        }
        visit::visit_item_use(self, node);
    }
}

fn extract_root_import(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(prefix) => extract_root_import(&prefix.tree),
        syn::UseTree::Name(name) => name.ident.to_string(),
        syn::UseTree::Rename(rename) => rename.rename.to_string(),
        syn::UseTree::Glob(_) => String::new(),
        syn::UseTree::Group(_) => String::new(),
    }
}

pub struct YAGNIDetector;

impl Default for YAGNIDetector {
    fn default() -> Self {
        Self
    }
}

impl YAGNIDetector {
    pub fn new() -> Self {
        Self
    }

    /// Analyze Rust code for YAGNI violations using syn AST.
    pub fn analyze(&self, code: &str) -> YAGNIReport {
        let syntax =
            syn::parse_file(code).unwrap_or_else(|_| syn::parse2(quote::quote!()).unwrap());
        let mut visitor = YagniVisitor::new();
        visitor.visit_file(&syntax);

        // Calculate call counts for functions
        let mut unused_functions = HashMap::new();
        for (name, mut info) in visitor.functions {
            let calls = *visitor.function_calls.get(&name).unwrap_or(&0);
            info.call_count = calls; // function_calls only counts calls, not definitions
            if info.call_count == 0 && !info.is_public && !info.is_test && name != "main" {
                unused_functions.insert(name, info);
            }
        }

        // Calculate variable usage
        let mut unused_variables = HashMap::new();
        for (name, mut info) in visitor.variables {
            let uses = *visitor.variable_uses.get(&name).unwrap_or(&0);
            info.usage_count = uses;
            if info.usage_count == 0 {
                unused_variables.insert(name, info);
            }
        }

        // Calculate unused imports (imported but not used in code)
        let mut unused_imports = HashSet::new();
        for imp in visitor.imports {
            // Check if the import is used anywhere in the code (not just in use statements)
            if !visitor.variable_uses.contains_key(&imp)
                && !visitor.function_calls.contains_key(&imp)
            {
                unused_imports.insert(imp);
            }
        }

        let score = ((unused_functions.len() + unused_variables.len() + unused_imports.len())
            as f64
            * 10.0)
            .min(100.0);

        YAGNIReport {
            unused_functions,
            unused_variables,
            unused_imports,
            over_engineering_score: score,
        }
    }
}

// Need quote for fallback empty file parsing
use quote;
