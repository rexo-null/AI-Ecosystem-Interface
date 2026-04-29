#![allow(dead_code)]

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, debug};

/// Symbol kind extracted from source code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Static,
    Type,
    Interface,
    Class,
    Variable,
    Import,
    Module,
    Unknown,
}

/// A symbol extracted from source code via Tree-sitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
}

/// Supported programming languages for indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Language::Rust,
            "ts" | "tsx" | "js" | "jsx" => Language::TypeScript,
            "py" | "pyw" => Language::Python,
            _ => Language::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Unknown => "unknown",
        }
    }
}

/// Semantic indexing entry for code/document search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub language: Language,
    pub symbols: Vec<CodeSymbol>,
    pub line_count: usize,
    pub byte_size: usize,
    pub indexed_at: i64,
}

/// Tree-sitter based code parser
struct CodeParser {
    rust_parser: tree_sitter::Parser,
    typescript_parser: tree_sitter::Parser,
    python_parser: tree_sitter::Parser,
}

impl CodeParser {
    fn new() -> Result<Self> {
        let mut rust_parser = tree_sitter::Parser::new();
        rust_parser
            .set_language(tree_sitter_rust::language())
            .context("Failed to set Rust language for Tree-sitter")?;

        let mut typescript_parser = tree_sitter::Parser::new();
        typescript_parser
            .set_language(tree_sitter_typescript::language_typescript())
            .context("Failed to set TypeScript language for Tree-sitter")?;

        let mut python_parser = tree_sitter::Parser::new();
        python_parser
            .set_language(tree_sitter_python::language())
            .context("Failed to set Python language for Tree-sitter")?;

        Ok(Self {
            rust_parser,
            typescript_parser,
            python_parser,
        })
    }

    /// Parse source code and extract symbols
    fn parse(&mut self, source: &str, language: Language) -> Vec<CodeSymbol> {
        let tree = match language {
            Language::Rust => self.rust_parser.parse(source, None),
            Language::TypeScript => self.typescript_parser.parse(source, None),
            Language::Python => self.python_parser.parse(source, None),
            Language::Unknown => return Vec::new(),
        };

        let tree = match tree {
            Some(t) => t,
            None => {
                warn!("Tree-sitter failed to parse source");
                return Vec::new();
            }
        };

        let root = tree.root_node();
        let mut symbols = Vec::new();
        self.extract_symbols(&root, source, language, &mut symbols);
        symbols
    }

    fn extract_symbols(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        language: Language,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        match language {
            Language::Rust => self.extract_rust_symbols(node, source, symbols),
            Language::TypeScript => self.extract_typescript_symbols(node, source, symbols),
            Language::Python => self.extract_python_symbols(node, source, symbols),
            Language::Unknown => {}
        }
    }

    fn extract_rust_symbols(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        let kind_str = node.kind();

        let symbol = match kind_str {
            "function_item" => self.extract_named_symbol(node, source, SymbolKind::Function),
            "struct_item" => self.extract_named_symbol(node, source, SymbolKind::Struct),
            "enum_item" => self.extract_named_symbol(node, source, SymbolKind::Enum),
            "trait_item" => self.extract_named_symbol(node, source, SymbolKind::Trait),
            "impl_item" => self.extract_impl_symbol(node, source),
            "const_item" => self.extract_named_symbol(node, source, SymbolKind::Const),
            "static_item" => self.extract_named_symbol(node, source, SymbolKind::Static),
            "type_item" => self.extract_named_symbol(node, source, SymbolKind::Type),
            "use_declaration" => self.extract_use_symbol(node, source),
            "mod_item" => self.extract_named_symbol(node, source, SymbolKind::Module),
            _ => None,
        };

        if let Some(sym) = symbol {
            symbols.push(sym);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_rust_symbols(&child, source, symbols);
        }
    }

    fn extract_typescript_symbols(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        let kind_str = node.kind();

        let symbol = match kind_str {
            "function_declaration" | "method_definition" | "arrow_function" => {
                self.extract_named_symbol(node, source, SymbolKind::Function)
            }
            "class_declaration" => {
                self.extract_named_symbol(node, source, SymbolKind::Class)
            }
            "interface_declaration" => {
                self.extract_named_symbol(node, source, SymbolKind::Interface)
            }
            "type_alias_declaration" => {
                self.extract_named_symbol(node, source, SymbolKind::Type)
            }
            "enum_declaration" => {
                self.extract_named_symbol(node, source, SymbolKind::Enum)
            }
            "import_statement" => {
                self.extract_import_symbol(node, source)
            }
            "lexical_declaration" | "variable_declaration" => {
                self.extract_variable_symbol(node, source)
            }
            _ => None,
        };

        if let Some(sym) = symbol {
            symbols.push(sym);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_typescript_symbols(&child, source, symbols);
        }
    }

    fn extract_python_symbols(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        let kind_str = node.kind();

        let symbol = match kind_str {
            "function_definition" => {
                self.extract_named_symbol(node, source, SymbolKind::Function)
            }
            "class_definition" => {
                self.extract_named_symbol(node, source, SymbolKind::Class)
            }
            "import_statement" | "import_from_statement" => {
                self.extract_import_symbol(node, source)
            }
            _ => None,
        };

        if let Some(sym) = symbol {
            symbols.push(sym);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_python_symbols(&child, source, symbols);
        }
    }

    /// Extract a symbol with a named child "name"
    fn extract_named_symbol(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        kind: SymbolKind,
    ) -> Option<CodeSymbol> {
        let name_node = node.child_by_field_name("name")?;
        let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();

        let start = node.start_position();
        let end = node.end_position();

        // Extract first line as signature
        let node_text = node.utf8_text(source.as_bytes()).ok()?;
        let signature = node_text.lines().next().map(|l| l.to_string());

        // Look for doc comment in previous sibling
        let doc_comment = self.extract_doc_comment(node, source);

        Some(CodeSymbol {
            name,
            kind,
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_col: start.column,
            end_col: end.column,
            signature,
            doc_comment,
        })
    }

    fn extract_impl_symbol(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<CodeSymbol> {
        let type_node = node.child_by_field_name("type")?;
        let name = type_node.utf8_text(source.as_bytes()).ok()?.to_string();

        let start = node.start_position();
        let end = node.end_position();
        let node_text = node.utf8_text(source.as_bytes()).ok()?;
        let signature = node_text.lines().next().map(|l| l.to_string());

        Some(CodeSymbol {
            name,
            kind: SymbolKind::Impl,
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_col: start.column,
            end_col: end.column,
            signature,
            doc_comment: None,
        })
    }

    fn extract_use_symbol(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<CodeSymbol> {
        let node_text = node.utf8_text(source.as_bytes()).ok()?.to_string();
        let start = node.start_position();
        let end = node.end_position();

        Some(CodeSymbol {
            name: node_text.clone(),
            kind: SymbolKind::Import,
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_col: start.column,
            end_col: end.column,
            signature: Some(node_text),
            doc_comment: None,
        })
    }

    fn extract_import_symbol(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<CodeSymbol> {
        let node_text = node.utf8_text(source.as_bytes()).ok()?.to_string();
        let start = node.start_position();
        let end = node.end_position();

        Some(CodeSymbol {
            name: node_text.clone(),
            kind: SymbolKind::Import,
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_col: start.column,
            end_col: end.column,
            signature: Some(node_text),
            doc_comment: None,
        })
    }

    fn extract_variable_symbol(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<CodeSymbol> {
        // Try to find the variable declarator child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = name_node.utf8_text(source.as_bytes()).ok()?.to_string();
                    let start = node.start_position();
                    let end = node.end_position();
                    let node_text = node.utf8_text(source.as_bytes()).ok()?;
                    let signature = node_text.lines().next().map(|l| l.to_string());

                    return Some(CodeSymbol {
                        name,
                        kind: SymbolKind::Variable,
                        start_line: start.row + 1,
                        end_line: end.row + 1,
                        start_col: start.column,
                        end_col: end.column,
                        signature,
                        doc_comment: None,
                    });
                }
            }
        }
        None
    }

    fn extract_doc_comment(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        let prev = node.prev_sibling()?;
        let text = prev.utf8_text(source.as_bytes()).ok()?;
        if prev.kind().contains("comment") && (text.starts_with("///") || text.starts_with("/**") || text.starts_with("#")) {
            Some(text.to_string())
        } else {
            None
        }
    }
}

/// Semantic Indexer - code and document indexing via Tree-sitter
pub struct SemanticIndexer {
    index: Arc<RwLock<HashMap<String, IndexEntry>>>,
    root_path: Option<PathBuf>,
    parser: Arc<tokio::sync::Mutex<CodeParser>>,
}

impl SemanticIndexer {
    pub fn new() -> Result<Self> {
        let parser = CodeParser::new()?;
        Ok(Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            root_path: None,
            parser: Arc::new(tokio::sync::Mutex::new(parser)),
        })
    }

    pub fn with_root(root_path: PathBuf) -> Result<Self> {
        let parser = CodeParser::new()?;
        Ok(Self {
            index: Arc::new(RwLock::new(HashMap::new())),
            root_path: Some(root_path),
            parser: Arc::new(tokio::sync::Mutex::new(parser)),
        })
    }

    /// Index a single file with Tree-sitter parsing
    pub async fn index_file(&self, path: &str, content: &str, language: Language) -> Result<String> {
        let id = format!("idx_{}", uuid::Uuid::new_v4());

        let symbols = {
            let mut parser = self.parser.lock().await;
            parser.parse(content, language)
        };

        debug!("Indexed {} symbols from {}", symbols.len(), path);

        let entry = IndexEntry {
            id: id.clone(),
            file_path: path.to_string(),
            content: content.to_string(),
            language,
            symbols,
            line_count: content.lines().count(),
            byte_size: content.len(),
            indexed_at: chrono::Utc::now().timestamp(),
        };

        self.index.write().await.insert(id.clone(), entry);
        Ok(id)
    }

    /// Index all supported files in a directory recursively
    pub async fn index_directory(&self, dir: &Path) -> Result<Vec<String>> {
        let mut indexed_ids = Vec::new();

        let entries = Self::walk_directory(dir).await?;
        info!("Found {} files to index in {:?}", entries.len(), dir);

        for file_path in entries {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let language = Language::from_extension(ext);
            if language == Language::Unknown {
                continue;
            }

            match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => {
                    let path_str = file_path.to_string_lossy().to_string();
                    match self.index_file(&path_str, &content, language).await {
                        Ok(id) => indexed_ids.push(id),
                        Err(e) => warn!("Failed to index {}: {}", path_str, e),
                    }
                }
                Err(e) => {
                    warn!("Failed to read {:?}: {}", file_path, e);
                }
            }
        }

        info!("Indexed {} files total", indexed_ids.len());
        Ok(indexed_ids)
    }

    /// Recursively walk a directory and collect file paths
    async fn walk_directory(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            let mut read_dir = tokio::fs::read_dir(&current).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Skip hidden dirs, node_modules, target, .git
                if file_name.starts_with('.')
                    || file_name == "node_modules"
                    || file_name == "target"
                    || file_name == "dist"
                    || file_name == "build"
                {
                    continue;
                }

                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    /// Search the index by keyword across file content and symbols
    pub async fn search(&self, query: &str, limit: usize) -> Vec<IndexEntry> {
        let index = self.index.read().await;
        let query_lower = query.to_lowercase();

        let mut results: Vec<IndexEntry> = index
            .values()
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query_lower)
                    || entry.file_path.to_lowercase().contains(&query_lower)
                    || entry.symbols.iter().any(|s| {
                        s.name.to_lowercase().contains(&query_lower)
                            || s.signature
                                .as_deref()
                                .map(|sig| sig.to_lowercase().contains(&query_lower))
                                .unwrap_or(false)
                    })
            })
            .cloned()
            .collect();

        results.truncate(limit);
        results
    }

    /// Search for specific symbol by name
    pub async fn search_symbols(&self, name: &str, kind: Option<SymbolKind>) -> Vec<(String, CodeSymbol)> {
        let index = self.index.read().await;
        let name_lower = name.to_lowercase();

        let mut results = Vec::new();

        for entry in index.values() {
            for symbol in &entry.symbols {
                let name_matches = symbol.name.to_lowercase().contains(&name_lower);
                let kind_matches = kind.as_ref().map_or(true, |k| &symbol.kind == k);

                if name_matches && kind_matches {
                    results.push((entry.file_path.clone(), symbol.clone()));
                }
            }
        }

        results
    }

    /// Get all symbols from a specific file
    pub async fn get_file_symbols(&self, file_path: &str) -> Vec<CodeSymbol> {
        let index = self.index.read().await;
        index
            .values()
            .find(|e| e.file_path == file_path)
            .map(|e| e.symbols.clone())
            .unwrap_or_default()
    }

    /// Remove a file from the index
    pub async fn remove_file(&self, path: &str) -> Result<()> {
        let mut index = self.index.write().await;
        index.retain(|_, entry| entry.file_path != path);
        info!("Removed from index: {}", path);
        Ok(())
    }

    /// Clear the entire index
    pub async fn clear(&self) {
        self.index.write().await.clear();
        info!("Index cleared");
    }

    /// Get the total number of indexed entries
    pub async fn count(&self) -> usize {
        self.index.read().await.len()
    }

    /// Get statistics about the index
    pub async fn stats(&self) -> IndexStats {
        let index = self.index.read().await;
        let mut total_symbols = 0;
        let mut total_lines = 0;
        let mut total_bytes = 0;
        let mut languages: HashMap<Language, usize> = HashMap::new();

        for entry in index.values() {
            total_symbols += entry.symbols.len();
            total_lines += entry.line_count;
            total_bytes += entry.byte_size;
            *languages.entry(entry.language).or_insert(0) += 1;
        }

        IndexStats {
            total_files: index.len(),
            total_symbols,
            total_lines,
            total_bytes,
            languages,
        }
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_symbols: usize,
    pub total_lines: usize,
    pub total_bytes: usize,
    pub languages: HashMap<Language, usize>,
}
